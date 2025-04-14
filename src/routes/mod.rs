pub mod health;
pub mod images;
pub mod preferences;
pub mod users;

use axum::{routing::get, Router};
use sqlx::PgPool;

pub fn create_router() -> Router<PgPool> {
    Router::new()
        // Health check route
        .route("/", get(health::health_check))
        // User routes
        .merge(users::routes())
        .merge(images::routes())
        .merge(preferences::routes())
}
