//! HTTP server and graceful shutdown.
//!
//! # Why no hyper / http crate
//!
//! nginx validates all HTTP protocol correctness from untrusted clients before
//! forwarding. The nginx → tsu connection is a trusted HTTP/1.1 stream, so a
//! simple line-oriented parser over a tokio `BufReader` is enough.
//!
//! # Required nginx setting: `proxy_buffering on`
//!
//! tsu only reads `Content-Length`-framed bodies. nginx with
//! `proxy_buffering on` (the default) always buffers the full client body and
//! forwards it to the backend with a `Content-Length` header. **Never set
//! `proxy_buffering off`** — doing so allows nginx to forward chunked bodies
//! that tsu cannot parse.
//!
//! # Keep-alive
//!
//! Connection lifetime is managed entirely by nginx, not tsu. nginx configured
//! with `proxy_http_version 1.1` and `proxy_set_header Connection ""` reuses
//! TCP connections to tsu based on its own `keepalive_timeout` and
//! `keepalive_requests` upstream settings. tsu loops until nginx closes the
//! connection (EOF) — it does not inspect the `Connection` header.
//!
//! # Graceful shutdown
//!
//! On SIGTERM/Ctrl-C the accept loop stops immediately. In-flight connection
//! tasks run to completion before `serve` returns. Set
//! `terminationGracePeriodSeconds` longer than your slowest request so k8s
//! doesn't SIGKILL before drain finishes.

use std::net::SocketAddr;
use std::sync::Arc;

use tokio::io::{AsyncBufReadExt, AsyncReadExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tracing::{error, info};

use crate::error::Error;
use crate::request::Request;
use crate::response::Response;
use crate::router::Router;
use crate::status::Status;

pub struct Server {
    addr: SocketAddr,
}

impl Server {
    /// Panics if `addr` is not a valid `host:port` string.
    pub fn bind(addr: &str) -> Self {
        let addr: SocketAddr = addr.parse().expect("invalid socket address");
        Self { addr }
    }

    /// Accepts connections and dispatches requests through `router`.
    /// Returns after a full graceful shutdown.
    pub async fn serve(self, router: Router) -> Result<(), Error> {
        let listener = TcpListener::bind(self.addr).await?;
        let router = Arc::new(router);

        info!(addr = %self.addr, "tsu listening");

        let mut tasks = tokio::task::JoinSet::new();
        let shutdown = shutdown_signal();
        tokio::pin!(shutdown);

        loop {
            tokio::select! {
                biased;

                () = &mut shutdown => {
                    info!(in_flight = tasks.len(), "shutdown signal received, draining connections");
                    break;
                }

                res = listener.accept() => {
                    let (stream, remote_addr) = match res {
                        Ok(v) => v,
                        Err(e) => { error!("accept error: {e}"); continue; }
                    };
                    let router = Arc::clone(&router);
                    tasks.spawn(async move {
                        if let Err(e) = serve_connection(stream, router).await {
                            error!(peer = %remote_addr, "connection error: {e}");
                        }
                    });
                }

                Some(_) = tasks.join_next(), if !tasks.is_empty() => {}
            }
        }

        while tasks.join_next().await.is_some() {}
        info!("tsu stopped");
        Ok(())
    }
}

// ── Connection handler ────────────────────────────────────────────────────────

/// Serves all requests on one TCP connection.
///
/// Loops until nginx closes the connection (EOF). nginx controls connection
/// lifetime via `keepalive_timeout` and `keepalive_requests` in the upstream
/// block — tsu never inspects the `Connection` header.
async fn serve_connection(stream: TcpStream, router: Arc<Router>) -> Result<(), Error> {
    let (read_half, mut write_half) = stream.into_split();
    let mut reader = BufReader::new(read_half);

    loop {
        // ── Request line ──────────────────────────────────────────────────────
        let mut line = String::new();
        if reader.read_line(&mut line).await? == 0 {
            break; // peer closed connection
        }
        let line = line.trim_end();
        let mut parts = line.splitn(3, ' ');
        let method = parts.next().unwrap_or("GET").to_uppercase();
        let path   = parts.next().unwrap_or("/").to_owned();
        // HTTP version field ignored — nginx guarantees HTTP/1.1

        // ── Headers ───────────────────────────────────────────────────────────
        let mut headers: Vec<(String, String)> = Vec::new();
        loop {
            let mut hline = String::new();
            reader.read_line(&mut hline).await?;
            let hline = hline.trim_end();
            if hline.is_empty() { break; }
            if let Some((name, value)) = hline.split_once(": ") {
                headers.push((name.to_owned(), value.to_owned()));
            }
        }

        // ── Body ──────────────────────────────────────────────────────────────
        let body = read_body(&mut reader, &headers).await?;

        // ── Dispatch ──────────────────────────────────────────────────────────
        let response = match router.lookup(&method, &path) {
            Some((handler, params)) => {
                handler.call(Request::new(method, path, headers, body, params)).await
            }
            None => Response::status(Status::NotFound),
        };

        response.write_to(&mut write_half).await?;
    }

    Ok(())
}

// ── Body readers ─────────────────────────────────────────────────────────────

async fn read_body<R: AsyncBufReadExt + Unpin>(
    reader: &mut R,
    headers: &[(String, String)],
) -> Result<Vec<u8>, Error> {
    if let Some(len) = headers.iter()
        .find(|(k, _)| k.eq_ignore_ascii_case("content-length"))
        .and_then(|(_, v)| v.trim().parse::<usize>().ok())
    {
        let mut buf = vec![0u8; len];
        reader.read_exact(&mut buf).await?;
        return Ok(buf);
    }

    Ok(Vec::new())
}

// ── Shutdown signal ───────────────────────────────────────────────────────────

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl-C handler");
    };

    #[cfg(unix)]
    let sigterm = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let sigterm = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c  => {}
        () = sigterm => {}
    }
}
