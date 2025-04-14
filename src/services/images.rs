use std::path::PathBuf;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use crate::error::AppError;
use crate::models::image::{Image, ImageData};

pub async fn build_image_data(
    patient_nb: &str,
    image: &Image,
    base_url: &str
) -> Result<ImageData, AppError> {
    let mut file_path = PathBuf::from(base_url);
    file_path.push(patient_nb);
    file_path.push(&image.filename);

    let canonical_path = file_path.canonicalize().map_err(|_| AppError::NotFound)?;
    let canonical_dir = PathBuf::from(base_url)
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

    let mut img_data = Vec::new();

    file.read_to_end(&mut img_data)
        .await
        .map_err(AppError::from)?;

    let content_type = match canonical_path.extension().and_then(|ext| ext.to_str()) {
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("png") => "image/png",
        Some("gif") => "image/gif",
        Some("svg") => "image/svg+xml",
        _ => "application/octet-stream",
    };

    let image_data = ImageData {
        image: image.clone(),
        content_type: Some(content_type.to_string()),
        data: Some(base64::encode(&img_data))
    };

    Ok(image_data)
}