//! Explicit, ephemeral HTTPS sharing. The ordinary desktop listener stays local.
use std::{
    net::{Ipv4Addr, SocketAddr, UdpSocket},
    sync::Arc,
    time::{Duration, Instant},
};

use axum::{
    Json, Router,
    extract::{ConnectInfo, State},
    http::{StatusCode, header},
    middleware,
    routing::{get, post},
};
use chrono::{DateTime, Utc};
use rand::{RngCore, rngs::OsRng};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;
use ts_rs::TS;

mod auth;
#[cfg(test)]
mod tests;
mod transport;

const PAIRING_TTL: Duration = Duration::from_secs(600);
const SESSION_TTL: Duration = Duration::from_secs(8 * 3600);
const COOKIE: &str = "__Host-openteams_lan";

#[derive(Debug, Deserialize, TS)]
pub struct LanEnableRequest {
    pub bind_address: String,
}

#[derive(Debug, Serialize, TS)]
pub struct LanStatus {
    pub enabled: bool,
    pub url: Option<String>,
    pub certificate_sha256: Option<String>,
    pub session_expires_at: Option<String>,
    pub suggested_bind_address: Option<String>,
}

#[derive(Debug, Serialize, TS)]
pub struct LanPairing {
    pub status: LanStatus,
    pub pairing_secret: String,
}

#[derive(Clone)]
pub struct LanController {
    app: Router,
    running: Arc<Mutex<Option<Running>>>,
}

struct Running {
    address: SocketAddr,
    fingerprint: String,
    auth: Arc<AuthState>,
    stop: CancellationToken,
}

struct AuthState {
    authority: String,
    stop: CancellationToken,
    credentials: Mutex<Credentials>,
}

struct Credentials {
    pairing_hash: Option<[u8; 32]>,
    pairing_deadline: Instant,
    session_hash: Option<[u8; 32]>,
    session_deadline: Option<Instant>,
    session_expires_at: Option<DateTime<Utc>>,
    attempts: Vec<Instant>,
    connections: CancellationToken,
}

impl LanController {
    pub fn new(app: Router) -> Self {
        Self {
            app,
            running: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn status(&self) -> LanStatus {
        let running = self.running.lock().await;
        let Some(active) = running
            .as_ref()
            .filter(|active| !active.stop.is_cancelled())
        else {
            return disabled_status();
        };
        let creds = active.auth.credentials.lock().await;
        LanStatus {
            enabled: true,
            url: Some(format!("https://{}", active.address)),
            certificate_sha256: Some(active.fingerprint.clone()),
            session_expires_at: creds.session_expires_at.map(|date| date.to_rfc3339()),
            suggested_bind_address: suggested_address(),
        }
    }

    pub async fn disable(&self) -> LanStatus {
        if let Some(active) = self.running.lock().await.take() {
            active.stop.cancel();
        }
        disabled_status()
    }

    async fn enable(&self, address: Ipv4Addr, allow_loopback: bool) -> Result<LanPairing, String> {
        if !(address.is_private() || allow_loopback && address.is_loopback()) {
            return Err(
                "Choose this Mac's private IPv4 address (10.x, 172.16–31.x, or 192.168.x).".into(),
            );
        }
        let mut running = self.running.lock().await;
        if running
            .as_ref()
            .is_some_and(|active| !active.stop.is_cancelled())
        {
            return Err(
                "LAN access is already enabled. Disable it before changing the address.".into(),
            );
        }
        let listener = tokio::net::TcpListener::bind((address, 0))
            .await
            .map_err(|e| format!("Cannot bind that local address: {e}"))?;
        let address = listener.local_addr().map_err(|e| e.to_string())?;
        let (tls, fingerprint) = transport::tls_config(address.ip())?;
        let secret = random_secret();
        let stop = CancellationToken::new();
        let auth = Arc::new(AuthState {
            authority: address.to_string(),
            stop: stop.clone(),
            credentials: Mutex::new(Credentials {
                pairing_hash: Some(hash(&secret)),
                pairing_deadline: Instant::now() + PAIRING_TTL,
                session_hash: None,
                session_deadline: None,
                session_expires_at: None,
                attempts: vec![],
                connections: stop.child_token(),
            }),
        });
        let app = self
            .app
            .clone()
            .layer(middleware::from_fn_with_state(auth.clone(), auth::gate));
        transport::spawn(listener, tls, app, auth.clone());
        let status = LanStatus {
            enabled: true,
            url: Some(format!("https://{address}")),
            certificate_sha256: Some(fingerprint.clone()),
            session_expires_at: None,
            suggested_bind_address: suggested_address(),
        };
        *running = Some(Running {
            address,
            fingerprint,
            auth,
            stop,
        });
        Ok(LanPairing {
            status,
            pairing_secret: secret,
        })
    }

    async fn rotate(&self) -> Result<LanPairing, String> {
        let running = self.running.lock().await;
        let active = running
            .as_ref()
            .filter(|active| !active.stop.is_cancelled())
            .ok_or("Enable LAN access first.")?;
        let secret = random_secret();
        let mut creds = active.auth.credentials.lock().await;
        creds.connections.cancel();
        creds.connections = active.stop.child_token();
        creds.pairing_hash = Some(hash(&secret));
        creds.pairing_deadline = Instant::now() + PAIRING_TTL;
        creds.session_hash = None;
        creds.session_deadline = None;
        creds.session_expires_at = None;
        creds.attempts.clear();
        Ok(LanPairing {
            status: LanStatus {
                enabled: true,
                url: Some(format!("https://{}", active.address)),
                certificate_sha256: Some(active.fingerprint.clone()),
                session_expires_at: None,
                suggested_bind_address: suggested_address(),
            },
            pairing_secret: secret,
        })
    }
}

fn disabled_status() -> LanStatus {
    LanStatus {
        enabled: false,
        url: None,
        certificate_sha256: None,
        session_expires_at: None,
        suggested_bind_address: suggested_address(),
    }
}

fn suggested_address() -> Option<String> {
    // UDP connect chooses a local route; no datagram is sent.
    let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)).ok()?;
    socket.connect((Ipv4Addr::new(192, 0, 2, 1), 9)).ok()?;
    let std::net::IpAddr::V4(ip) = socket.local_addr().ok()?.ip() else {
        return None;
    };
    ip.is_private().then(|| ip.to_string())
}

fn random_secret() -> String {
    let mut bytes = [0u8; 32];
    OsRng.fill_bytes(&mut bytes);
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
fn hash(value: &str) -> [u8; 32] {
    Sha256::digest(value.as_bytes()).into()
}
fn same_hash(a: &[u8; 32], b: &[u8; 32]) -> bool {
    a.iter().zip(b).fold(0u8, |diff, (a, b)| diff | (a ^ b)) == 0
}

pub fn control_routes(controller: LanController) -> Router {
    Router::new()
        .route("/api/lan", get(status))
        .route("/api/lan/enable", post(enable))
        .route("/api/lan/disable", post(disable))
        .route("/api/lan/rotate", post(rotate))
        .layer(middleware::from_fn(local_only))
        .with_state(controller)
}

async fn local_only(
    request: axum::extract::Request,
    next: middleware::Next,
) -> axum::response::Response {
    let is_local_peer = request
        .extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .is_some_and(|peer| peer.0.ip().is_loopback());
    let host = request
        .headers()
        .get(header::HOST)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<axum::http::uri::Authority>().ok());
    let local_host = host.as_ref().is_some_and(|host| {
        host.host() == "localhost"
            || host
                .host()
                .trim_matches(['[', ']'])
                .parse::<std::net::IpAddr>()
                .is_ok_and(|ip| ip.is_loopback())
    });
    let origin_ok = request
        .headers()
        .get(header::ORIGIN)
        .map(|value| {
            value.to_str().is_ok_and(|origin| {
                host.as_ref()
                    .is_some_and(|host| origin == format!("http://{host}"))
            })
        })
        .unwrap_or(true);
    if !is_local_peer || !local_host || !origin_ok {
        return auth::error(
            StatusCode::FORBIDDEN,
            "LAN settings are available only on this Mac.",
        );
    }
    next.run(request).await
}

async fn status(State(controller): State<LanController>) -> Json<LanStatus> {
    Json(controller.status().await)
}
async fn enable(
    State(controller): State<LanController>,
    Json(request): Json<LanEnableRequest>,
) -> Result<Json<LanPairing>, (StatusCode, String)> {
    let ip = request.bind_address.parse::<Ipv4Addr>().map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            "A private IPv4 address is required.".into(),
        )
    })?;
    controller
        .enable(ip, false)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::BAD_REQUEST, e))
}
async fn disable(State(controller): State<LanController>) -> Json<LanStatus> {
    Json(controller.disable().await)
}
async fn rotate(
    State(controller): State<LanController>,
) -> Result<Json<LanPairing>, (StatusCode, String)> {
    controller
        .rotate()
        .await
        .map(Json)
        .map_err(|e| (StatusCode::BAD_REQUEST, e))
}
