# Changelog

All notable changes to astor are documented here.

Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
astor adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [Unreleased]

---

## [0.1.0] — 2026-02-25

First release. The foundation is here. Radix-tree routing, raw HTTP/1.1 parsing, graceful shutdown, health probes — and nothing the reverse proxy already handles.

### Added

- `ContentType` enum — `Csv`, `EventStream`, `FormData`, `Html`, `Json`, `MsgPack`, `OctetStream`, `Pdf`, `Text`, `Xml`.
- `health` module — built-in `health::liveness` (`/healthz`) and `health::readiness` (`/readyz`) for Kubernetes probes.
- `IntoResponse` trait — return your own types directly from handlers.
- `Request` — path parameters (`req.param`), method, URI, headers, raw body bytes (`req.body() -> &[u8]`).
- `Response` — shortcut constructors (`Response::json`, `Response::text`, `Response::status`) and a typed builder (`Response::builder().status(...).header(...).json(...)`).
- `Router` — radix-tree routing via `matchit`. `GET`, `POST`, `PUT`, `PATCH`, `DELETE`, and arbitrary methods.
- `Server::bind` — graceful shutdown on `SIGTERM` / `Ctrl-C`. Waits for in-flight requests to drain.
- `Status` enum — every IANA-registered HTTP status code as a named variant.
- Production nginx configuration (`nginx/nginx.conf`) and Kubernetes manifests (`k8s/`).
- Raw tokio HTTP/1.1 parsing — no hyper, no http crate.

[Unreleased]: https://github.com/benjaminPla/astor/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/benjaminPla/astor/releases/tag/v0.1.0
