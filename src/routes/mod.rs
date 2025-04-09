pub(crate) mod health;
pub(crate) mod users;

use axum::{Router, routing::get};
use sqlx::PgPool;

pub fn create_router() -> Router<PgPool> {
    Router::new()
        // Health check route
        .route("/", get(health::root))
        // User routes
        .merge(users::routes())
}
