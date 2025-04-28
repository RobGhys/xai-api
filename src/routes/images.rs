use axum::{
    extract::{Path, State}, http::{header, StatusCode},
    response::IntoResponse,
    routing::get,
    Json,
    Router,
};
use serde_json::to_string_pretty;
use sqlx::PgPool;
use std::path::PathBuf;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use once_cell::sync::Lazy;
use serde::de::Unexpected::Option;
use crate::enums::mask_type::MaskType;
use crate::error::AppError;
use crate::models::image::{Image, ImageData, ImageDataSet, ImageSet};
use crate::models::mask::{Mask, MaskImage};
use crate::services;
use crate::services::images::process_patient_dirs;

const BASE_URL: Lazy<String> = Lazy::new(|| {
    std::env::var("IMAGES_PATH")
        .unwrap_or_else(|_| "/Users/rob/projects/xai_lung_tum/xp_data_paper".to_string())
});

pub fn routes() -> Router<PgPool> {
    Router::new()
        .route("/images/id/{id}", get(get_image_by_id))
        .route("/images/patient/{patient_nb}", get(get_images_by_patient_nb))
        .route("/images", get(get_images))
        .route("/images/{id}/set", get(get_image_with_masks))
        .route("/file/{patient_nb}/{filename}", get(serve_file))
        .route("/images/process", get(process_all_patients))
}

#[utoipa::path(
    get,
    path = "/images",
    responses(
        (status = 200, description = "List of all existing images", body = Vec<Image>),
        (status = 500, description = "Internal Server Error")
    ),
    tag = "images"
)]
pub async fn get_images(State(pool): State<PgPool>) -> Result<Json<Vec<Image>>, AppError> {
    let images = sqlx::query_as!(Image, "SELECT id, filename, patient_nb FROM images")
        .fetch_all(&pool)
        .await
        .map_err(AppError::from)?;

    tracing::info!("Found images: {}", images.len());

    Ok(Json(images))
}

#[utoipa::path(
    get,
    path = "/images/patient/{patient_nb}",
    responses(
        (status = 200, description = "List of all existing images for a given patient", body = Vec<Image>),
        (status = 500, description = "Internal Server Error")
    ),
    tag = "images"
)]
pub async fn get_images_by_patient_nb(
    State(pool): State<PgPool>,
    Path(patient_nb): Path<String>,
) -> Result<Json<Vec<Image>>, AppError> {
    let images = sqlx::query_as!(
        Image,
        "SELECT id, filename, patient_nb
FROM images
WHERE patient_nb = $1",
    patient_nb)
        .fetch_all(&pool)
        .await
        .map_err(AppError::from)?;

    tracing::info!("Found images: {}", images.len());

    Ok(Json(images))
}

#[utoipa::path(
    get,
    path = "/images/id/{id}",
    responses(
        (status = 200, description = "Image Found", body = Image),
        (status = 404, description = "Image Not Found"),
        (status = 500, description = "Internal Server Error")
    ),
    tag = "images"
)]
pub async fn get_image_by_id(
    State(pool): State<PgPool>,
    Path(id): Path<i32>,
) -> Result<Json<Image>, AppError> {
    let image = sqlx::query_as!(
        Image,
        "SELECT id, filename, patient_nb FROM images WHERE id = $1",
        id
    )
    .fetch_optional(&pool)
    .await
    .map_err(AppError::from)?
    .ok_or_else(|| AppError::NotFound)?;

    tracing::info!("Image found: {}", to_string_pretty(&image).unwrap());

    Ok(Json(image))
}

#[utoipa::path(
    get,
    path = "/images/{patient_nb}/set",
    responses(
        (status = 200, description = "Found image with masks", body = ImageSet),
        (status = 404, description = "Image wit masks not found"),
        (status = 500, description = "Internal Server Error")
    ),
    tag = "images"
)]
pub async fn get_image_with_masks(
    State(pool): State<PgPool>,
    Path(patient_nb): Path<i32>,
) -> Result<Json<ImageSet>, AppError> {
    let images = sqlx::query_as!(
        Image,
        "SELECT id, filename, patient_nb FROM images WHERE id = $1",
        patient_nb
    )
    .fetch_all(&pool)
    .await
    .map_err(AppError::from)?;

    // Get the first image only. We only handle 1 image per patient
    let image = &images[0];

    let masks = sqlx::query_as!(
        Mask,
        r#"SELECT id, image_id, mask_type as "mask_type!: MaskType", filename FROM masks WHERE image_id = $1"#,
        image.id
    )
        .fetch_all(&pool)
        .await
        .map_err(AppError::from)?;

    let image_set = ImageSet {
        original_image: format!("/file/{}/{}", image.patient_nb, image.filename),
        masks: masks
            .into_iter()
            .map(|mask| MaskImage {
                id: mask.id.to_string(),
                r#type: format!("{:?}", mask.mask_type).to_lowercase(),
                image_url: format!("file/{}/{}", image.patient_nb, mask.filename),
                content_type: None,
                image_data: None,
            })
            .collect(),
    };

    tracing::info!(
        "Found Images Set: {}",
        to_string_pretty(&image_set).unwrap()
    );

    Ok(Json(image_set))
}

#[utoipa::path(
    get,
    path = "/file/{patient_nb}/{filename}",
    responses(
        (status = 200, description = "Fichier trouvé"),
        (status = 404, description = "Fichier non trouvé"),
        (status = 500, description = "Erreur interne du serveur")
    ),
    tag = "files"
)]
pub async fn serve_file(Path((patient_nb, filename)): Path<(String, String)>,
) -> Result<impl IntoResponse, AppError> {
    // &: dereference Lazy<String>; *: get ref to the String
    let mut file_path = PathBuf::from(&*BASE_URL);
    file_path.push(&patient_nb);
    file_path.push(&filename);

    let canonical_path = file_path.canonicalize().map_err(|_| AppError::NotFound)?;
    // &: dereference Lazy<String>; *: get ref to the String
    let canonical_dir = PathBuf::from(&*BASE_URL)
        .canonicalize()
        .map_err(AppError::from)?;

    // protection against directory traversal attacks
    if !canonical_path.starts_with(canonical_dir) {
        return Err(AppError::BadRequest(
            "Invalid file path".to_string(),
        ));
    }

    let mut file = File::open(&canonical_path)
        .await
        .map_err(|_| AppError::NotFound)?;
    let mut contents = Vec::new();
    file.read_to_end(&mut contents)
        .await
        .map_err(AppError::from)?;

    let content_type = match canonical_path.extension().and_then(|ext| ext.to_str()) {
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("png") => "image/png",
        Some("gif") => "image/gif",
        Some("svg") => "image/svg+xml",
        _ => "application/octet-stream",
    };

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, content_type)],
        contents,
    ))
}

#[utoipa::path(
    get,
    path = "/images/process",
    responses(
        (status = 200, description = "Processed patient directories successfully"),
        (status = 500, description = "Internal Server Error")
    ),
    tag = "images"
)]
pub async fn process_all_patients(
    State(pool): State<PgPool>,
) -> Result<impl IntoResponse, AppError> {
    process_patient_dirs(State(pool),  &*BASE_URL).await?;

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({ "status": "success", "message": "Patient directories processed successfully" }))
    ))
}