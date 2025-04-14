/// Get a hello world message
#[utoipa::path(
    get,
    path = "/",
    responses(
        (status = 200, description = "Health message returned successfully", body = String)
    ),
    tag = "health"
)]
pub async fn health_check() -> &'static str {
    "API is running!"
}
