use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

// the input to our `create_user` handler
#[derive(Deserialize, ToSchema)]
pub struct CreateUser {
    /// The username of the user
    pub username: String,
}

// the output to our `create_user` handler
#[derive(Serialize, sqlx::FromRow, ToSchema)]
pub struct User {
    /// The user's unique identifier
    pub id: i32,
    /// The username of the user
    pub username: String,
}
