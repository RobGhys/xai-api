mod api;
mod config;
mod db;
mod error;
mod models;
mod routes;

use api::openapi::{ApiDoc, create_swagger_ui};
use config::Config;
use db::connection::establish_connection;
use db::migrations::run_migrations;
use dotenv::dotenv;
use routes::create_router;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

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
    run_migrations(&pool).await;

    // build our application with a route
    let app = create_router()
        // Add Swagger UI
        .merge(create_swagger_ui())
        .layer(TraceLayer::new_for_http())
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
