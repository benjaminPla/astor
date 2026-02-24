//! tsu example — covers every Response variant and common handler patterns.
//!
//! Run:
//!   RUST_LOG=info cargo run --example basic
//!
//! Try:
//!   curl http://localhost:3000/users/42
//!   curl -X POST http://localhost:3000/users \
//!        -H 'content-type: application/json' -d '{"name":"alice"}'
//!   curl -X PATCH http://localhost:3000/users/42 \
//!        -H 'content-type: application/json' -d '{"name":"bob"}'
//!   curl -X DELETE http://localhost:3000/users/42
//!   curl http://localhost:3000/xml
//!   curl http://localhost:3000/redirect
//!   curl http://localhost:3000/healthz
//!   curl http://localhost:3000/readyz

use tsu::{ContentType, Request, Response, Router, Server, health};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let app = Router::new()
        // CRUD
        .get("/users/:id",    get_user)
        .post("/users",       create_user)
        .patch("/users/:id",  update_user)
        .delete("/users/:id", delete_user)
        // Other response types
        .get("/xml",      xml_response)
        .get("/redirect", redirect)
        // Health probes — always register these for k8s
        .get("/healthz",  health::liveness)
        .get("/readyz",   health::readiness);

    Server::bind("0.0.0.0:3000").serve(app).await.expect("server error");
}

// ── GET /users/:id ────────────────────────────────────────────────────────────
//
// Response::json takes Vec<u8> — pass bytes from your serialiser directly.
//   serde_json:  Response::json(serde_json::to_vec(&user).unwrap())
//   hand-built:  format!(...).into_bytes()  ← zero-cost, no copy
async fn get_user(req: Request) -> Response {
    let id = req.param("id").unwrap_or("unknown");
    Response::json(format!(r#"{{"id":"{id}","name":"alice"}}"#).into_bytes())
}

// ── POST /users ───────────────────────────────────────────────────────────────
//
// req.body() is &[u8]. Parse with serde_json::from_slice, simd-json, etc.
// tsu does not touch the bytes.
// 201 Created + Location header.
async fn create_user(req: Request) -> Response {
    if req.body().is_empty() {
        return Response::status(400);
    }
    // Real app: let input: CreateUser = serde_json::from_slice(req.body()).unwrap();
    Response::builder()
        .status(201)
        .header("location", "/users/99")
        .json(r#"{"id":"99","name":"new_user"}"#.to_owned().into_bytes())
}

// ── PATCH /users/:id ─────────────────────────────────────────────────────────
//
// 200 with updated resource. Builder with no extra headers — same as shortcut.
async fn update_user(req: Request) -> Response {
    let id = req.param("id").unwrap_or("unknown");
    Response::json(format!(r#"{{"id":"{id}","name":"updated"}}"#).into_bytes())
}

// ── DELETE /users/:id ─────────────────────────────────────────────────────────
//
// 204 No Content — no body, no content-type.
async fn delete_user(_req: Request) -> Response {
    Response::status(204)
}

// ── GET /xml ──────────────────────────────────────────────────────────────────
//
// Non-JSON body via ContentType enum.
// Same pattern works for Html, Csv, Pdf, OctetStream, MsgPack, EventStream.
async fn xml_response(_req: Request) -> Response {
    Response::builder()
        .status(200)
        .bytes(ContentType::Xml, b"<users><user id=\"1\"/></users>".to_vec())
}

// ── GET /redirect ─────────────────────────────────────────────────────────────
//
// 301 redirect — custom status + header, no body.
async fn redirect(_req: Request) -> Response {
    Response::builder()
        .status(301)
        .header("location", "/users/1")
        .no_body()
}
