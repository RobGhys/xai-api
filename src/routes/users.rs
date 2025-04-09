use axum::{Json, Router, extract::State, http::StatusCode, routing::post};
use serde_json::to_string_pretty;
use sqlx::PgPool;

use crate::error::AppError;
use crate::models::user::{CreateUser, User};

pub fn routes() -> Router<PgPool> {
    Router::new().route("/users", post(create_user).get(get_users))
}

/// Create a new user
#[utoipa::path(
    post,
    path = "/users",
    request_body = CreateUser,
    responses(
        (status = 201, description = "User created successfully", body = User),
        (status = 500, description = "Internal server error")
    ),
    tag = "users"
)]
pub async fn create_user(
    State(pool): State<PgPool>,
    Json(payload): Json<CreateUser>,
) -> Result<(StatusCode, Json<User>), AppError> {
    let user = sqlx::query_as!(
        User,
        "INSERT INTO users (username) VALUES ($1) RETURNING id, username",
        payload.username
    )
    .fetch_one(&pool)
    .await
    .map_err(AppError::from)?;

    tracing::info!("User created: {}", to_string_pretty(&user).unwrap());

    Ok((StatusCode::CREATED, Json(user)))
}

/// Get all users
#[utoipa::path(
    get,
    path = "/users",
    responses(
        (status = 200, description = "List all users", body = Vec<User>),
        (status = 500, description = "Internal server error")
    ),
    tag = "users"
)]
pub async fn get_users(State(pool): State<PgPool>) -> Result<Json<Vec<User>>, AppError> {
    let users = sqlx::query_as!(User, "SELECT id as \"id: i32\", username FROM users")
        .fetch_all(&pool)
        .await
        .map_err(AppError::from)?;

    tracing::info!("Users found: {}", to_string_pretty(&users).unwrap());

    Ok(Json(users))
}
