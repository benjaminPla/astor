//! Minimal tsu example — CRUD-style JSON endpoints and health checks.
//!
//! Run with:
//!   RUST_LOG=info cargo run --example basic
//!
//! Try:
//!   curl http://localhost:3000/users/42
//!   curl -X POST http://localhost:3000/users \
//!        -H 'content-type: application/json' \
//!        -d '{"name":"alice"}'
//!   curl -X DELETE http://localhost:3000/users/42
//!   curl http://localhost:3000/healthz

use tsu::{Request, Response, Router, Server, health};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let app = Router::new()
        .get("/users/:id",    get_user)
        .post("/users",       create_user)
        .delete("/users/:id", delete_user)
        .get("/healthz",      health::liveness)
        .get("/readyz",       health::readiness);

    Server::bind("0.0.0.0:3000")
        .serve(app)
        .await
        .expect("server error");
}

// GET /users/:id
//
// Response::json takes Vec<u8> — pass bytes from your serialiser:
//   serde_json:  Response::json(serde_json::to_vec(&user).unwrap())
//   hand-built:  Response::json(format!(...).into_bytes())  ← zero-cost, no copy
async fn get_user(req: Request) -> Response {
    let id = req.param("id").unwrap_or("unknown");
    Response::json(format!(r#"{{"id":"{id}","name":"alice"}}"#).into_bytes())
}

// POST /users
//
// req.body() is &[u8] — parse with serde_json::from_slice, simd-json, etc.
// tsu does not touch the bytes.
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

// DELETE /users/:id → 204 No Content
async fn delete_user(_req: Request) -> Response {
    Response::status(204)
}
