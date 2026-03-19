use axum::{routing::get, Json, Router};
use shared::config::AppConfig;
use tower_http::cors::CorsLayer;
use tracing_subscriber::EnvFilter;

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "ok" }))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let config = AppConfig::from_env()?;

    let cors = CorsLayer::new()
        .allow_origin(
            config
                .allowed_origin
                .parse::<http::HeaderValue>()
                .expect("valid ALLOWED_ORIGIN"),
        )
        .allow_methods([
            http::Method::GET,
            http::Method::POST,
            http::Method::PUT,
            http::Method::PATCH,
            http::Method::DELETE,
        ])
        .allow_headers([http::header::CONTENT_TYPE, http::header::AUTHORIZATION])
        .allow_credentials(true);

    let app = Router::new()
        .route("/api/health", get(health))
        .layer(cors);

    let addr = format!("0.0.0.0:{}", config.port);
    tracing::info!("Starting server on {}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
