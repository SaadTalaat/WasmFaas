use axum::{
    extract::Extension,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use faas::{
    handlers::{DeployHandler, InvokeHandler, WSHandler},
    state::Handles,
    compiler::Compiler,
    Registry,
    Settings,
};

use std::error::Error;
use std::net::SocketAddr;
use tower::ServiceBuilder;
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
    trace::TraceLayer,
};

enum MyError {}

impl IntoResponse for MyError {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::BAD_REQUEST, "Bad Request").into_response()
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let settings = Settings::new()?;
    tracing::info!("Running with settings: {:?}", settings);
    tracing_subscriber::fmt::init();
    let registry = Registry::start(settings.registry);
    let compiler = Compiler::new("./boilerplate");
    let handles = Handles::new(registry, compiler)?;
    let extra_layers = ServiceBuilder::new()
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::new().allow_origin(Any))
        .layer(Extension(handles));
    // allow requests from any origin

    let app = Router::new()
        .nest_service("/assets", ServeDir::new("bin"))
        .route("/", get(|| async { "hello" }))
        .route("/deploy", post(DeployHandler))
        .route("/invoke/:name", post(InvokeHandler))
        .route("/ws", get(WSHandler))
        .layer(extra_layers);

    let listen_addr: SocketAddr = "0.0.0.0:8090".parse()?;
    tracing::info!("Server started: Listening on: {}", listen_addr);
    axum::Server::bind(&listen_addr)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await?;
    Ok(())
}
