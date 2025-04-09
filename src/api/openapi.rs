use crate::models::user::{CreateUser, User};
use crate::routes::{health, users};
use utoipa::OpenApi;
use utoipa_swagger_ui::{Config, SwaggerUi};

#[derive(OpenApi)]
#[openapi(
    paths(
        health::root,
        users::create_user,
        users::get_users
    ),
    components(
        schemas(CreateUser, User)
    ),
    tags(
        (name = "health", description = "Health check endpoints"),
        (name = "users", description = "User management endpoints"),
        (name = "xai-api", description = "API for the Lungs-xAI Research Project")
    )
)]
pub struct ApiDoc;

pub fn create_swagger_ui() -> SwaggerUi {
    let config = Config::from("/api-docs/openapi.json")
        .use_base_layout()
        .try_it_out_enabled(true);

    SwaggerUi::new("/swagger-ui")
        .config(config)
        .url("/api-docs/openapi.json", ApiDoc::openapi())
}
