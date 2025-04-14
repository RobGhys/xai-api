mod api;
mod config;
mod db;
mod enums;
mod error;
mod models;
mod routes;
mod services;

use api::openapi::{create_swagger_ui};
use config::Config;
use db::connection::establish_connection;
use dotenv::dotenv;
use routes::create_router;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use tower_http::cors::{CorsLayer, Any};
use axum::http::{Method, HeaderValue};
use axum::http::header::CONTENT_TYPE;


#[tokio::main]
async fn main() {
    // .env
    dotenv().ok();
    let config = Config::from_env();

    // initialize tracing
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| config.log_level.clone().into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting application...");

    // DB connection
    let pool = establish_connection(&config).await;

    // CORS
    let cors = CorsLayer::new()
        .allow_origin("http://localhost:5173".parse::<HeaderValue>().unwrap()) // Vite
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers([CONTENT_TYPE]);

    // build our application with a route
    let app = create_router()
        // Add Swagger UI
        .merge(create_swagger_ui())
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        // Add DB pool
        .with_state(pool);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind(&config.server_addr)
        .await
        .unwrap();
    tracing::info!("Server running on http://{}", config.server_addr);
    tracing::info!(
        "Swagger UI available at http://{}/swagger-ui/",
        config.server_addr
    );

    axum::serve(listener, app).await.unwrap();
}
