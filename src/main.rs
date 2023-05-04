use axum::{
    extract::{Extension, Path},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use faas::handlers::{DeployHandler, InvokeHandler, WSHandler};
use faas::state::Handles;
use faas::Registry;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{self, Write};
use std::net::SocketAddr;
use std::process::Command;
use tempfile::{Builder, TempDir};

enum MyError {}

impl IntoResponse for MyError {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::BAD_REQUEST, "Bad Request").into_response()
    }
}

#[tokio::main]
async fn main() {
    let registry = Registry::start();
    let handles = Handles::new(registry);
    let app = Router::new()
        .route("/", get(|| async { "hello" }))
        .route("/deploy", post(DeployHandler))
        .route("/invoke/:name", post(InvokeHandler))
        .route("/ws", get(WSHandler))
        .layer(Extension(handles));
    println!("Listening");
    axum::Server::bind(&"0.0.0.0:8090".parse().unwrap())
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .unwrap();
}
