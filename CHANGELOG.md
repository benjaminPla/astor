# Changelog

All notable changes to astor are documented here.

Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
astor adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [Unreleased]

### Added

- `Method` enum — all RFC 9110 standard methods (`Connect`, `Delete`, `Get`, `Head`, `Options`, `Patch`, `Post`, `Put`, `Trace`), WebDAV extensions (`Copy`, `Lock`, `Mkcalendar`, `Mkcol`, `Move`, `Propfind`, `Proppatch`, `Report`, `Search`, `Unlock`), and `Purge` (nginx / Varnish cache invalidation).

### Changed

- `Request::method()` now returns `Method` instead of `&str`.
- `Router::route()` now takes `Method` instead of `&str` — passing an arbitrary string is a compile error.
- Unknown method strings are rejected with `405 Method Not Allowed` at the connection level, before any handler runs. Note: nginx does not block unknown methods by default — configure `limit_except` in your nginx location block to enforce this at the proxy layer too (see `nginx/nginx.conf`).

---

## [0.1.1] — 2026-02-25

### Fixed

- Path parameters (`/users/:id`) now match correctly. matchit 0.8 switched from `:param` to `{param}` syntax; astor now translates at registration time so user-facing API is unchanged.

### Removed

- `tracing` dependency — astor is a library; consumers bring their own logging. Errors surface via `Result`.

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

[Unreleased]: https://github.com/benjaminPla/astor/compare/v0.1.1...HEAD
[0.1.1]: https://github.com/benjaminPla/astor/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/benjaminPla/astor/releases/tag/v0.1.0
