use std::{
    future::Future,
    io,
    net::IpAddr,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use axum::Router;
use hyper_util::{
    rt::{TokioExecutor, TokioIo},
    server::conn::auto::Builder,
    service::TowerToHyperService,
};
use rustls::{ServerConfig, pki_types::PrivatePkcs8KeyDer};
use sha2::{Digest, Sha256};
use tokio::{
    io::{AsyncRead, AsyncWrite, ReadBuf},
    net::{TcpListener, TcpStream},
};
use tokio_rustls::TlsAcceptor;
use tokio_util::sync::{CancellationToken, WaitForCancellationFutureOwned};

pub(super) fn tls_config(ip: IpAddr) -> Result<(Arc<ServerConfig>, String), String> {
    let key = rcgen::KeyPair::generate().map_err(|e| e.to_string())?;
    let mut params =
        rcgen::CertificateParams::new(vec![ip.to_string()]).map_err(|e| e.to_string())?;
    params
        .distinguished_name
        .push(rcgen::DnType::CommonName, "OpenTeams temporary LAN access");
    let cert = params.self_signed(&key).map_err(|e| e.to_string())?;
    let fingerprint = Sha256::digest(cert.der().as_ref())
        .iter()
        .map(|byte| format!("{byte:02X}"))
        .collect::<Vec<_>>()
        .join(":");
    let config = ServerConfig::builder_with_provider(Arc::new(
        rustls::crypto::aws_lc_rs::default_provider(),
    ))
    .with_safe_default_protocol_versions()
    .map_err(|e| e.to_string())?
    .with_no_client_auth()
    .with_single_cert(
        vec![cert.der().clone()],
        PrivatePkcs8KeyDer::from(key.serialize_der()).into(),
    )
    .map_err(|e| e.to_string())?;
    Ok((Arc::new(config), fingerprint))
}

pub(super) fn spawn(
    listener: TcpListener,
    config: Arc<ServerConfig>,
    app: Router,
    auth: Arc<super::AuthState>,
) {
    let stop = auth.stop.clone();
    tokio::spawn(async move {
        let acceptor = TlsAcceptor::from(config);
        // Bound handshakes/connections; no unbounded task creation on a shared LAN.
        let slots = Arc::new(tokio::sync::Semaphore::new(32));
        loop {
            let accepted = tokio::select! { biased; _ = stop.cancelled() => break, accepted = listener.accept() => accepted };
            let Ok((stream, _peer)) = accepted else {
                break;
            };
            let Ok(permit) = slots.clone().try_acquire_owned() else {
                continue;
            };
            let acceptor = acceptor.clone();
            let app = app.clone();
            let stop = auth.credentials.lock().await.connections.clone();
            tokio::spawn(async move {
                let io = RevocableIo::new(stream, stop.clone(), permit);
                let handshake = tokio::select! {
                    _ = stop.cancelled() => return,
                    result = tokio::time::timeout(std::time::Duration::from_secs(10), acceptor.accept(io)) => result,
                };
                let Ok(Ok(tls)) = handshake else {
                    return;
                };
                // Cancellation lives underneath TLS and remains attached after an HTTP
                // upgrade, so detached WebSocket tasks cannot outlive revocation.
                let builder = Builder::new(TokioExecutor::new());
                let connection = builder.serve_connection_with_upgrades(
                    TokioIo::new(tls),
                    TowerToHyperService::new(app),
                );
                tokio::select! { _ = stop.cancelled() => {}, _ = connection => {} }
            });
        }
    });
}

pub(super) struct RevocableIo {
    stream: TcpStream,
    // The upgraded IO owns the permit, rather than the short-lived HTTP task.
    _permit: tokio::sync::OwnedSemaphorePermit,
    cancelled: Pin<Box<WaitForCancellationFutureOwned>>,
}
impl RevocableIo {
    pub(super) fn new(
        stream: TcpStream,
        token: CancellationToken,
        permit: tokio::sync::OwnedSemaphorePermit,
    ) -> Self {
        Self {
            stream,
            _permit: permit,
            cancelled: Box::pin(token.cancelled_owned()),
        }
    }
    fn check(&mut self, cx: &mut Context<'_>) -> io::Result<()> {
        if self.cancelled.as_mut().poll(cx).is_ready() {
            Err(io::Error::new(
                io::ErrorKind::ConnectionAborted,
                "LAN access revoked",
            ))
        } else {
            Ok(())
        }
    }
}
impl AsyncRead for RevocableIo {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        self.check(cx)?;
        Pin::new(&mut self.stream).poll_read(cx, buf)
    }
}
impl AsyncWrite for RevocableIo {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        self.check(cx)?;
        Pin::new(&mut self.stream).poll_write(cx, buf)
    }
    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        self.check(cx)?;
        Pin::new(&mut self.stream).poll_flush(cx)
    }
    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.stream).poll_shutdown(cx)
    }
}
