use axum::Router;
use sqlx::PgPool;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::models::{
    image::{Image, ImageSet},
    mask::{Mask, MaskImage},
    preference::{Preference, PreferenceEvent, PreferenceWithEvents},
    user::User,
};
use crate::routes::{health, images, preferences, users};

#[derive(OpenApi)]
#[openapi(
    paths(
        health::health_check,
        users::create_user,
        users::get_users,
        images::get_images,
        images::get_image_by_id,
        images::process_all_patients,
        images::get_images_by_patient_nb,
        images::get_image_with_masks,
        images::serve_file,
        preferences::create_preference,
        preferences::get_preferences,
        preferences::get_preference,
        preferences::get_next_image
    ),
    components(
        schemas(
            User,
            crate::models::user::CreateUser,
            Image,
            Mask,
            ImageSet,
            MaskImage,
            Preference,
            PreferenceEvent,
            crate::models::preference::CreatePreference,
            crate::models::preference::CreatePreferenceEvent,
            PreferenceWithEvents,
            crate::models::preference::RankedMask
        )
    ),
    tags(
        (name = "health", description = "Health check endpoints"),
        (name = "users", description = "User management endpoints"),
        (name = "images", description = "Image management endpoints"),
        (name = "masks", description = "Mask management endpoints"),
        (name = "files", description = "File serving endpoints"),
        (name = "preferences", description = "Preference management endpoints")
    ),
    info(
        title = "XAI API",
        version = "0.1.0",
        description = "API pour servir des images et collecter des préférences d'utilisateurs",
    )
)]
pub struct ApiDoc;

pub fn create_swagger_ui() -> Router<PgPool> {
    SwaggerUi::new("/swagger-ui")
        .url("/api-docs/openapi.json", ApiDoc::openapi())
        .into()
}
