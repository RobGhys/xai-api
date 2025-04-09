/// Get a hello world message
#[utoipa::path(
    get,
    path = "/",
    responses(
        (status = 200, description = "Health message returned successfully", body = String)
    ),
    tag = "health"
)]
pub async fn root() -> &'static str {
    "API is running!"
}
