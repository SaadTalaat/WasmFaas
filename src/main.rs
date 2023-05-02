use axum::{Router, Json, routing::{get, post}, extract::{Path, Extension}, response::IntoResponse, http::StatusCode};
use tempfile::{Builder, TempDir};
use std::process::Command;
use serde::{Serialize, Deserialize};
use std::fs::File;
use std::io::{self, Write};
use faas::handlers::{WSHandler, DeployHandler, InvokeHandler};
use faas::state::Handles;
use std::net::SocketAddr;
enum MyError {
}

impl IntoResponse for MyError {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::BAD_REQUEST, "Bad Request").into_response()
    }
}

#[tokio::main]
async fn main() {
    let handles = Handles::new();
    let app = Router::new()
        .route("/", get(|| async {"hello"}))
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
