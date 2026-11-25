use axum::{
    body::Body,
    extract::ws::{Message, WebSocketUpgrade},
    http::{Method, Request},
    response::IntoResponse,
};
use futures_util::StreamExt;
use tower::ServiceExt;

use super::*;

fn fixture() -> (Arc<AuthState>, Router, String) {
    let secret = "a".repeat(64);
    let auth = Arc::new(AuthState {
        authority: "192.168.1.2:4433".into(),
        stop: CancellationToken::new(),
        credentials: Mutex::new(Credentials {
            pairing_hash: Some(hash(&secret)),
            pairing_deadline: Instant::now() + PAIRING_TTL,
            session_hash: None,
            session_deadline: None,
            session_expires_at: None,
            attempts: vec![],
            connections: CancellationToken::new(),
        }),
    });
    let app = Router::new()
        .fallback(|| async { "protected" })
        .layer(middleware::from_fn_with_state(auth.clone(), auth::gate));
    (auth, app, secret)
}

fn request(
    method: Method,
    path: &str,
    cookie: Option<&str>,
    origin: Option<&str>,
) -> axum::http::request::Builder {
    let mut builder = Request::builder()
        .method(method)
        .uri(path)
        .header(header::HOST, "192.168.1.2:4433");
    if let Some(cookie) = cookie {
        builder = builder.header(header::COOKIE, cookie);
    }
    if let Some(origin) = origin {
        builder = builder.header(header::ORIGIN, origin);
    }
    builder
}

async fn login(app: &Router, secret: &str) -> axum::response::Response {
    app.clone()
        .oneshot(
            request(
                Method::POST,
                "/lan/login",
                None,
                Some("https://192.168.1.2:4433"),
            )
            .body(Body::from(format!("secret={secret}")))
            .unwrap(),
        )
        .await
        .unwrap()
}

#[tokio::test]
async fn unauthenticated_rest_sse_and_websocket_are_denied() {
    let (_, app, _) = fixture();
    for path in [
        "/api/chat/sessions",
        "/api/events",
        "/api/chat/sessions/1/stream",
    ] {
        let mut builder = request(Method::GET, path, None, Some("https://192.168.1.2:4433"));
        if path.ends_with("stream") {
            builder = builder.header(header::UPGRADE, "websocket");
        }
        let response = app
            .clone()
            .oneshot(builder.body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED, "{path}");
    }
}

#[tokio::test]
async fn pairing_is_single_use_secure_and_rate_limited() {
    let (_, app, secret) = fixture();
    let response = login(&app, &secret).await;
    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    let cookie = response.headers()[header::SET_COOKIE].to_str().unwrap();
    for flag in ["Secure", "HttpOnly", "SameSite=Strict", "Path=/"] {
        assert!(cookie.contains(flag));
    }
    assert_eq!(
        login(&app, &secret).await.status(),
        StatusCode::UNAUTHORIZED
    );
    for _ in 0..3 {
        assert_eq!(login(&app, "x").await.status(), StatusCode::UNAUTHORIZED);
    }
    assert_eq!(
        login(&app, &secret).await.status(),
        StatusCode::TOO_MANY_REQUESTS
    );
}

#[tokio::test]
async fn expired_pairing_and_session_are_rejected() {
    let (auth, app, secret) = fixture();
    auth.credentials.lock().await.pairing_deadline = Instant::now() - Duration::from_secs(1);
    assert_eq!(
        login(&app, &secret).await.status(),
        StatusCode::UNAUTHORIZED
    );
    auth.credentials.lock().await.pairing_deadline = Instant::now() + PAIRING_TTL;
    let response = login(&app, &secret).await;
    let cookie = response.headers()[header::SET_COOKIE]
        .to_str()
        .unwrap()
        .split(';')
        .next()
        .unwrap();
    auth.credentials.lock().await.session_deadline = Some(Instant::now() - Duration::from_secs(1));
    let response = app
        .oneshot(
            request(Method::GET, "/api/events", Some(cookie), None)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn exact_host_origin_and_local_action_boundaries_are_enforced() {
    let (_, app, secret) = fixture();
    let response = login(&app, &secret).await;
    let cookie = response.headers()[header::SET_COOKIE]
        .to_str()
        .unwrap()
        .split(';')
        .next()
        .unwrap();
    for (method, origin, upgrade) in [
        (Method::POST, None, false),
        (Method::POST, Some("https://evil.test"), false),
        (Method::GET, None, true),
        (Method::GET, Some("https://evil.test"), true),
    ] {
        let mut builder = request(method, "/api/workflow", Some(cookie), origin);
        if upgrade {
            builder = builder.header(header::UPGRADE, "websocket");
        }
        assert_eq!(
            app.clone()
                .oneshot(builder.body(Body::empty()).unwrap())
                .await
                .unwrap()
                .status(),
            StatusCode::FORBIDDEN
        );
    }
    let wrong_host = Request::builder()
        .uri("/api/events")
        .header(header::HOST, "evil.test")
        .header(header::COOKIE, cookie)
        .body(Body::empty())
        .unwrap();
    assert_eq!(
        app.clone().oneshot(wrong_host).await.unwrap().status(),
        StatusCode::FORBIDDEN
    );
    for path in [
        "/api/lan",
        "/api/lan/enable",
        "/api/filesystem/select-directory",
        "/api/filesystem/open-in-explorer",
        "/api/version/restart",
        "/api/admin/migrate-projects",
        "/api/browser-session",
        "/api/output-validation",
    ] {
        assert_eq!(
            app.clone()
                .oneshot(
                    request(Method::GET, path, Some(cookie), None)
                        .body(Body::empty())
                        .unwrap()
                )
                .await
                .unwrap()
                .status(),
            StatusCode::FORBIDDEN,
            "{path}"
        );
    }
    assert_eq!(
        app.oneshot(
            request(Method::GET, "/api/events", Some(cookie), None)
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap()
        .status(),
        StatusCode::OK
    );
}

#[tokio::test]
async fn logout_invalidates_session_and_revokes_transport() {
    let (auth, app, secret) = fixture();
    let response = login(&app, &secret).await;
    let cookie = response.headers()[header::SET_COOKIE]
        .to_str()
        .unwrap()
        .split(';')
        .next()
        .unwrap();
    let connections = auth.credentials.lock().await.connections.clone();
    let response = app
        .clone()
        .oneshot(
            request(
                Method::POST,
                "/lan/logout",
                Some(cookie),
                Some("https://192.168.1.2:4433"),
            )
            .body(Body::empty())
            .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
    assert!(auth.credentials.lock().await.session_hash.is_none());
    assert_eq!(
        app.oneshot(
            request(Method::GET, "/api/events", Some(cookie), None)
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap()
        .status(),
        StatusCode::UNAUTHORIZED
    );
    tokio::time::timeout(Duration::from_secs(1), connections.cancelled())
        .await
        .unwrap();
    assert!(!auth.stop.is_cancelled());
}

#[tokio::test]
async fn default_is_disabled_and_production_rejects_loopback_wildcard_public() {
    let controller = LanController::new(Router::new());
    assert!(!controller.status().await.enabled);
    for address in [
        "127.0.0.1",
        "0.0.0.0",
        "8.8.8.8",
        "169.254.1.1",
        "224.0.0.1",
    ] {
        assert!(
            controller
                .enable(address.parse().unwrap(), false)
                .await
                .is_err()
        );
    }
}

#[tokio::test]
async fn local_controls_require_loopback_peer_host_and_post_for_mutation() {
    let controller = LanController::new(Router::new());
    let app = control_routes(controller.clone());
    let mut absent = Request::builder()
        .uri("/api/lan")
        .header(header::HOST, "localhost:3000")
        .body(Body::empty())
        .unwrap();
    assert_eq!(
        app.clone().oneshot(absent).await.unwrap().status(),
        StatusCode::FORBIDDEN
    );
    for method in [Method::GET, Method::HEAD] {
        absent = Request::builder()
            .method(method)
            .uri("/api/lan/enable")
            .header(header::HOST, "localhost:3000")
            .body(Body::empty())
            .unwrap();
        absent
            .extensions_mut()
            .insert(ConnectInfo("127.0.0.1:5555".parse::<SocketAddr>().unwrap()));
        assert_eq!(
            app.clone().oneshot(absent).await.unwrap().status(),
            StatusCode::METHOD_NOT_ALLOWED
        );
    }
    assert!(!controller.status().await.enabled);
}

#[tokio::test]
async fn real_https_login_websocket_revocation_rotate_and_disable() {
    let ws = |upgrade: WebSocketUpgrade| async move {
        upgrade
            .on_upgrade(|mut socket| async move {
                while let Some(Ok(message)) = socket.recv().await {
                    if let Message::Text(text) = message {
                        let _ = socket.send(Message::Text(text)).await;
                    }
                }
            })
            .into_response()
    };
    let app = Router::new()
        .route("/api/ws", get(ws))
        .route("/api/health", get(|| async { "ok" }));
    let controller = LanController::new(app);
    let pairing = controller.enable(Ipv4Addr::LOCALHOST, true).await.unwrap();
    let url = pairing.status.url.clone().unwrap();
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();
    assert_eq!(
        client
            .get(format!("{url}/api/health"))
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::UNAUTHORIZED
    );
    let login = client
        .post(format!("{url}/lan/login"))
        .header(header::ORIGIN, &url)
        .body(format!("secret={}", pairing.pairing_secret))
        .send()
        .await
        .unwrap();
    assert_eq!(login.status(), StatusCode::SEE_OTHER);
    let cookie = login.headers()[header::SET_COOKIE]
        .to_str()
        .unwrap()
        .split(';')
        .next()
        .unwrap()
        .to_string();
    assert_eq!(
        client
            .get(format!("{url}/api/health"))
            .header(header::COOKIE, &cookie)
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::OK
    );
    let response = client
        .get(format!("{url}/api/ws"))
        .header(header::ORIGIN, &url)
        .header(header::COOKIE, &cookie)
        .header(header::CONNECTION, "upgrade")
        .header(header::UPGRADE, "websocket")
        .header("sec-websocket-version", "13")
        .header("sec-websocket-key", "dGhlIHNhbXBsZSBub25jZQ==")
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::SWITCHING_PROTOCOLS);
    let mut socket = response.upgrade().await.unwrap();
    let rotated = controller.rotate().await.unwrap();
    use tokio::io::AsyncReadExt;
    let mut byte = [0u8; 1];
    let end = tokio::time::timeout(Duration::from_secs(2), socket.read(&mut byte))
        .await
        .unwrap();
    assert!(
        matches!(end, Ok(0) | Err(_)),
        "Upgraded transport survived revocation"
    );
    assert_ne!(pairing.pairing_secret, rotated.pairing_secret);
    let next_url = rotated.status.url.unwrap();
    assert_eq!(
        client
            .get(format!("{next_url}/api/health"))
            .header(header::COOKIE, cookie)
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::UNAUTHORIZED
    );
    controller.disable().await;
    assert!(!controller.status().await.enabled);
    tokio::time::sleep(Duration::from_millis(30)).await;
    assert!(
        client
            .get(format!("{next_url}/api/health"))
            .send()
            .await
            .is_err()
    );
}

#[tokio::test]
async fn real_https_logout_closes_stream_but_keeps_pairing_page_reachable() {
    let app = Router::new().route(
        "/api/events",
        get(|| async {
            let stream =
                futures_util::stream::once(async { Ok::<_, std::io::Error>("data: ready\n\n") })
                    .chain(futures_util::stream::pending());
            axum::response::Response::builder()
                .header(header::CONTENT_TYPE, "text/event-stream")
                .body(Body::from_stream(stream))
                .unwrap()
        }),
    );
    let controller = LanController::new(app);
    let pairing = controller.enable(Ipv4Addr::LOCALHOST, true).await.unwrap();
    let url = pairing.status.url.unwrap();
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();
    let login = client
        .post(format!("{url}/lan/login"))
        .header(header::ORIGIN, &url)
        .body(format!("secret={}", pairing.pairing_secret))
        .send()
        .await
        .unwrap();
    let cookie = login.headers()[header::SET_COOKIE]
        .to_str()
        .unwrap()
        .split(';')
        .next()
        .unwrap()
        .to_string();
    let mut stream = client
        .get(format!("{url}/api/events"))
        .header(header::COOKIE, &cookie)
        .send()
        .await
        .unwrap()
        .bytes_stream();
    assert_eq!(stream.next().await.unwrap().unwrap(), "data: ready\n\n");
    // A successful response can race synchronous transport revocation.
    let _ = client
        .post(format!("{url}/lan/logout"))
        .header(header::ORIGIN, &url)
        .header(header::COOKIE, &cookie)
        .send()
        .await;
    let end = tokio::time::timeout(Duration::from_secs(2), stream.next())
        .await
        .unwrap();
    assert!(end.is_none() || end.is_some_and(|chunk| chunk.is_err()));
    assert!(controller.status().await.enabled);
    assert_eq!(
        client.get(format!("{url}/")).send().await.unwrap().status(),
        StatusCode::OK
    );
    assert_eq!(
        client
            .get(format!("{url}/api/events"))
            .header(header::COOKIE, &cookie)
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::UNAUTHORIZED
    );
    let next = controller.rotate().await.unwrap();
    assert_eq!(next.status.url.as_deref(), Some(url.as_str()));
    // Rotation closes every previous transport, including idle keep-alive
    // connections. A newly loaded pairing page starts a fresh connection.
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();
    let login = client
        .post(format!("{url}/lan/login"))
        .header(header::ORIGIN, &url)
        .body(format!("secret={}", next.pairing_secret))
        .send()
        .await
        .unwrap();
    let cookie = login.headers()[header::SET_COOKIE]
        .to_str()
        .unwrap()
        .split(';')
        .next()
        .unwrap()
        .to_string();
    let mut stream = client
        .get(format!("{url}/api/events"))
        .header(header::COOKIE, &cookie)
        .send()
        .await
        .unwrap()
        .bytes_stream();
    assert!(stream.next().await.unwrap().is_ok());
    controller.disable().await;
    let end = tokio::time::timeout(Duration::from_secs(2), stream.next())
        .await
        .unwrap();
    assert!(end.is_none() || end.is_some_and(|chunk| chunk.is_err()));
}

#[tokio::test]
async fn pairing_consumption_is_atomic_under_concurrent_requests() {
    let (_, app, secret) = fixture();
    let (first, second) = tokio::join!(login(&app, &secret), login(&app, &secret));
    let mut statuses = [first.status().as_u16(), second.status().as_u16()];
    statuses.sort();
    assert_eq!(statuses, [303, 401]);
}

#[tokio::test]
async fn real_local_control_http_receives_connect_info() {
    let controller = LanController::new(Router::new());
    let app =
        control_routes(controller.clone()).into_make_service_with_connect_info::<SocketAddr>();
    let listener = tokio::net::TcpListener::bind((Ipv4Addr::LOCALHOST, 0))
        .await
        .unwrap();
    let address = listener.local_addr().unwrap();
    let stop = CancellationToken::new();
    let cancelled = stop.clone();
    let server = tokio::spawn(async move {
        axum::serve(listener, app)
            .with_graceful_shutdown(cancelled.cancelled_owned())
            .await
            .unwrap();
    });
    let response = reqwest::get(format!("http://{address}/api/lan"))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let status: serde_json::Value = response.json().await.unwrap();
    assert_eq!(status["enabled"], false);
    assert!(status["url"].is_null());
    stop.cancel();
    server.await.unwrap();
}
