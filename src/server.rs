//! HTTP server and graceful shutdown.
//!
//! # Graceful shutdown and Kubernetes
//!
//! When Kubernetes terminates a pod it sends **SIGTERM** and waits
//! `terminationGracePeriodSeconds` (default 30 s) before sending SIGKILL.
//!
//! The server reacts by:
//! 1. Immediately stopping `listener.accept()` — no new connections are made.
//! 2. Letting every in-flight connection task run to completion.
//! 3. Returning from [`Server::serve`], which lets `main` exit cleanly.
//!
//! Set `terminationGracePeriodSeconds` in your pod spec to a value longer
//! than your slowest request. 30 s is a reasonable default for most APIs.

use std::net::SocketAddr;
use std::sync::Arc;

use hyper::service::service_fn;
use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server::conn::auto::Builder as ConnBuilder;
use tokio::net::TcpListener;
use tracing::{error, info};

use crate::error::Error;
use crate::request::Request;
use crate::response::Response;
use crate::router::Router;

/// The HTTP server.
pub struct Server {
    addr: SocketAddr,
}

impl Server {
    /// Configures the server to bind to `addr` when [`serve`](Server::serve)
    /// is called.
    ///
    /// # Panics
    ///
    /// Panics if `addr` is not a valid `host:port` string.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use tsu::Server;
    /// let server = Server::bind("0.0.0.0:3000");
    /// ```
    pub fn bind(addr: &str) -> Self {
        let addr: SocketAddr = addr.parse().expect("invalid socket address");
        Self { addr }
    }

    /// Starts accepting connections and dispatching them through `router`.
    ///
    /// Returns only after a full graceful shutdown (SIGTERM or Ctrl-C,
    /// followed by all in-flight requests completing).
    pub async fn serve(self, router: Router) -> Result<(), Error> {
        let listener = TcpListener::bind(self.addr).await?;

        // Wrap router in Arc so it can be shared across concurrent connection
        // tasks without copying the entire routing table.
        let router = Arc::new(router);

        info!(addr = %self.addr, "tsu listening");

        // JoinSet tracks every spawned connection task so we can wait for
        // them all to finish during graceful shutdown.
        let mut tasks = tokio::task::JoinSet::new();

        // Pin the shutdown future so we can poll it in a loop.
        // Futures in Rust must not move in memory after the first poll — that
        // is what `Pin` enforces. `tokio::pin!` pins the future on the stack.
        let shutdown = shutdown_signal();
        tokio::pin!(shutdown);

        loop {
            tokio::select! {
                // `biased` makes select! check arms top-to-bottom instead of
                // randomly. We check shutdown first so a SIGTERM immediately
                // stops accepting new connections, even if more are queued.
                biased;

                () = &mut shutdown => {
                    info!(in_flight = tasks.len(), "shutdown signal received, draining connections");
                    break;
                }

                res = listener.accept() => {
                    let (stream, remote_addr) = match res {
                        Ok(v) => v,
                        Err(e) => {
                            error!("accept error: {e}");
                            continue;
                        }
                    };

                    let router = Arc::clone(&router);
                    // TokioIo adapts tokio's AsyncRead/AsyncWrite to the hyper
                    // IO traits.
                    let io = TokioIo::new(stream);

                    tasks.spawn(async move {
                        // `service_fn` turns a plain async function into a
                        // hyper `Service`. The closure is called once per
                        // request on the connection, not once per connection.
                        let svc = service_fn(move |req| {
                            let router = Arc::clone(&router);
                            async move { dispatch(router, req, remote_addr).await }
                        });

                        // `auto::Builder` transparently handles both HTTP/1.1
                        // and HTTP/2 — whatever the client negotiates.
                        if let Err(e) = ConnBuilder::new(TokioExecutor::new())
                            .serve_connection(io, svc)
                            .await
                        {
                            error!(peer = %remote_addr, "connection error: {e}");
                        }
                    });
                }

                // Reap finished connection tasks so the JoinSet does not grow
                // without bound on long-running servers.
                Some(_) = tasks.join_next(), if !tasks.is_empty() => {}
            }
        }

        // Drain: wait for every in-flight connection to finish before we return.
        while tasks.join_next().await.is_some() {}

        info!("tsu stopped");
        Ok(())
    }
}

// ── Request dispatch ──────────────────────────────────────────────────────────

/// Core hot path: routes one request and produces one response.
///
/// The error type is [`Infallible`](std::convert::Infallible) — we handle all
/// failures internally (returning 404, 500, etc.) so hyper never sees an error.
async fn dispatch(
    router: Arc<Router>,
    req: hyper::Request<hyper::body::Incoming>,
    _remote_addr: std::net::SocketAddr,
) -> Result<http::Response<http_body_util::Full<bytes::Bytes>>, std::convert::Infallible> {
    let method = req.method().clone();
    let path = req.uri().path().to_owned();

    let response = match router.lookup(&method, &path) {
        Some((handler, params)) => handler.call(Request::new(req, params)).await,
        None => Response::status(http::StatusCode::NOT_FOUND),
    };

    Ok(response.into_inner())
}

// ── Shutdown signal ───────────────────────────────────────────────────────────

/// Resolves on the first shutdown signal the process receives.
///
/// On Unix this listens for both **SIGTERM** (sent by `kubectl` and the
/// Kubernetes control plane) and **SIGINT** (Ctrl-C, for local dev).
/// On Windows only Ctrl-C is available.
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

    // `pending()` is a future that never resolves — on non-Unix platforms
    // the SIGTERM arm is effectively disabled.
    #[cfg(not(unix))]
    let sigterm = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c   => {}
        () = sigterm  => {}
    }
}
