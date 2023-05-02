use crate::compile::compile;
use crate::status::Status;
use crate::state::Handles;
use std::io::Write;
use serde::Deserialize;
use axum::{
    Json,
    extract::Extension,
    response::IntoResponse,
    http::StatusCode
};

#[derive(Deserialize)]
pub struct DeployableFunction {
    name: String,
    body: String
}

pub async fn deploy(Extension(handles): Extension<Handles>, Json(func): Json<DeployableFunction>) -> impl IntoResponse {
    let bytes = compile(&func.body).await.unwrap();
    println!("Deploying: {}", bytes.len());
    handles.storage.store(&func.name, &bytes);
    Json(Status::ok())
}

async fn store_func(func: &String, bytes: &Vec<u8>) {
    let mut fd = std::fs::File::create(format!("bin/{}.wasm", func)).unwrap();
    fd.write(bytes).unwrap();

}
