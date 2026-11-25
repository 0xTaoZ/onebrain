use base64::Engine as _;
use std::{sync::Arc, time::Instant};

use axum::{
    body::{Body, to_bytes},
    extract::{Request, State},
    http::{Method, StatusCode, header},
    middleware::Next,
    response::{IntoResponse, Response},
};

use super::{AuthState, COOKIE, SESSION_TTL, hash, random_secret, same_hash};

pub(super) async fn gate(
    State(auth): State<Arc<AuthState>>,
    request: Request,
    next: Next,
) -> Response {
    if auth.stop.is_cancelled() {
        return error(
            StatusCode::UNAUTHORIZED,
            "LAN access has expired or been revoked.",
        );
    }
    let host_ok = request
        .headers()
        .get(header::HOST)
        .and_then(|v| v.to_str().ok())
        == Some(auth.authority.as_str());
    if !host_ok {
        return error(StatusCode::FORBIDDEN, "Unexpected host.");
    }
    let expected_origin = format!("https://{}", auth.authority);
    let origin = request
        .headers()
        .get(header::ORIGIN)
        .and_then(|v| v.to_str().ok());
    let upgrade = request.headers().get(header::UPGRADE).is_some();
    let safe = matches!(
        *request.method(),
        Method::GET | Method::HEAD | Method::OPTIONS
    );
    if origin.is_some_and(|value| value != expected_origin)
        || ((!safe || upgrade) && origin != Some(expected_origin.as_str()))
    {
        return error(StatusCode::FORBIDDEN, "Same-origin requests are required.");
    }
    let path = request.uri().path();
    if blocked(path) {
        return error(
            StatusCode::FORBIDDEN,
            "This action is available only on the Mac.",
        );
    }
    if path == "/lan/login" && request.method() == Method::POST {
        return pair(auth, request).await;
    }
    let cookie = request
        .headers()
        .get_all(header::COOKIE)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .flat_map(|line| line.split(';'))
        .filter_map(|entry| entry.trim().split_once('='))
        .filter(|(name, _)| *name == COOKIE)
        .map(|(_, value)| value)
        .collect::<Vec<_>>();
    let authorized = if cookie.len() == 1 && cookie[0].len() == 64 {
        let creds = auth.credentials.lock().await;
        creds
            .session_deadline
            .is_some_and(|deadline| deadline > Instant::now())
            && creds
                .session_hash
                .as_ref()
                .is_some_and(|stored| same_hash(stored, &hash(cookie[0])))
    } else {
        false
    };
    if !authorized {
        if path.starts_with("/api/") || path == "/api" || upgrade {
            return error(
                StatusCode::UNAUTHORIZED,
                "Pair this browser from the desktop app first.",
            );
        }
        return login_form();
    }
    if path == "/lan/logout" && request.method() == Method::POST {
        let mut creds = auth.credentials.lock().await;
        creds.session_hash = None;
        creds.session_deadline = None;
        creds.session_expires_at = None;
        creds.connections.cancel();
        creds.connections = auth.stop.child_token();
        drop(creds);
        return Response::builder()
            .status(StatusCode::NO_CONTENT)
            .header(
                header::SET_COOKIE,
                format!("{COOKIE}=; Path=/; Secure; HttpOnly; SameSite=Strict; Max-Age=0"),
            )
            .header(header::CACHE_CONTROL, "no-store")
            .body(Body::empty())
            .unwrap();
    }
    let mut response = next.run(request).await;
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, "no-store".parse().unwrap());
    response
        .headers_mut()
        .insert(header::REFERRER_POLICY, "no-referrer".parse().unwrap());
    response
}

fn blocked(path: &str) -> bool {
    let path = path.trim_end_matches('/');
    path == "/api/lan"
        || path.starts_with("/api/lan/")
        || path == "/api/admin"
        || path.starts_with("/api/admin/")
        || matches!(
            path,
            "/api/filesystem/select-directory"
                | "/api/filesystem/open-in-explorer"
                | "/api/version/update-npx"
                | "/api/version/restart"
                | "/api/browser-session"
                | "/api/output-validation"
        )
}

async fn pair(auth: Arc<AuthState>, request: Request) -> Response {
    let now = Instant::now();
    let mut creds = auth.credentials.lock().await;
    creds
        .attempts
        .retain(|attempt| now.duration_since(*attempt).as_secs() < 60);
    if creds.attempts.len() >= 5 {
        return error(
            StatusCode::TOO_MANY_REQUESTS,
            "Too many attempts. Wait one minute.",
        );
    }
    creds.attempts.push(now);
    drop(creds);
    let body = match tokio::time::timeout(
        std::time::Duration::from_secs(5),
        to_bytes(request.into_body(), 4096),
    )
    .await
    {
        Ok(Ok(body)) => body,
        _ => return error(StatusCode::BAD_REQUEST, "Invalid pairing request."),
    };
    let fields = url::form_urlencoded::parse(&body)
        .filter(|(key, _)| key == "secret")
        .map(|(_, value)| value.into_owned())
        .collect::<Vec<_>>();
    // Reacquire after the bounded body read; consumption remains atomic while
    // slow clients cannot hold up disable/rotate/status.
    let mut creds = auth.credentials.lock().await;
    let now = Instant::now();
    if fields.len() != 1
        || fields[0].len() != 64
        || now >= creds.pairing_deadline
        || !creds
            .pairing_hash
            .as_ref()
            .is_some_and(|stored| same_hash(stored, &hash(&fields[0])))
    {
        return error(
            StatusCode::UNAUTHORIZED,
            "Invalid, expired, or already used pairing code.",
        );
    }
    let secret = random_secret();
    creds.pairing_hash = None;
    creds.session_hash = Some(hash(&secret));
    creds.session_deadline = Some(now + SESSION_TTL);
    creds.session_expires_at =
        Some(chrono::Utc::now() + chrono::Duration::seconds(SESSION_TTL.as_secs() as i64));
    drop(creds);
    let issued_hash = hash(&secret);
    tokio::spawn(async move {
        tokio::select! {
            _ = auth.stop.cancelled() => {},
            _ = tokio::time::sleep(SESSION_TTL) => {
                let mut creds = auth.credentials.lock().await;
                if creds.session_hash == Some(issued_hash) {
                    creds.session_hash = None;
                    creds.session_deadline = None;
                    creds.session_expires_at = None;
                    creds.connections.cancel();
                    creds.connections = auth.stop.child_token();
                }
            }
        }
    });
    Response::builder()
        .status(StatusCode::SEE_OTHER)
        .header(header::LOCATION, "/")
        .header(header::CACHE_CONTROL, "no-store")
        .header(
            header::SET_COOKIE,
            format!(
                "{COOKIE}={secret}; Path=/; Secure; HttpOnly; SameSite=Strict; Max-Age={}",
                SESSION_TTL.as_secs()
            ),
        )
        .body(Body::empty())
        .unwrap()
}

fn login_form() -> Response {
    let css = include_str!("login.css");
    let style_hash = base64::engine::general_purpose::STANDARD.encode(hash(css));
    let html = include_str!("login.html").replace("{{CSS}}", css);
    Response::builder()
        .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
        .header(header::CACHE_CONTROL, "no-store")
        .header(header::REFERRER_POLICY, "no-referrer")
        .header("content-security-policy", format!("default-src 'none'; style-src 'sha256-{style_hash}'; form-action 'self'; frame-ancestors 'none'; base-uri 'none'"))
        .body(Body::from(html))
        .unwrap()
}

pub(super) fn error(status: StatusCode, message: &str) -> Response {
    let mut response = (status, axum::Json(serde_json::json!({"error": message}))).into_response();
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, "no-store".parse().unwrap());
    if status == StatusCode::UNAUTHORIZED {
        response.headers_mut().insert("x-openteams-lan-auth", "required".parse().unwrap());
    }
    response
}
