use std::fs;
use std::path::{Path, PathBuf};
use axum::extract::State;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use crate::error::AppError;
use crate::models::image::{Image, ImageData};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use sqlx::PgPool;
use crate::enums::mask_type::MaskType;

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
        data: Some(STANDARD.encode(&img_data))
    };

    Ok(image_data)
}

pub async fn get_latest_patient_nb(pool: &PgPool) -> Result<Option<String>, AppError> {
    // get latest patient_id from db
    let latest_patient_nb: Option<i32> = sqlx::query_scalar!(
        "SELECT MAX(CAST(patient_nb AS INTEGER)) FROM images"
        )
        .fetch_one(pool)
        .await
        .map_err(AppError::from)?;

    // shadowing
    let latest_patient_str: Option<String> = latest_patient_nb.map(|nb| format!("{:03}", nb));

    Ok((latest_patient_str))
}

pub async fn get_patient_dirs_to_process(
    pool: &PgPool,
    base_url: &str
) -> Result<Vec<PathBuf>, AppError> {
    let latest_patient_str = get_latest_patient_nb(pool).await?;
    tracing::info!("Latest patient_nb in DB: {:?}", latest_patient_str);
    
    // browse all images by parsing patient_nb
    let file_path = PathBuf::from(base_url);
    let dirs = fs::read_dir(&file_path).map_err(AppError::from)?;

    let mut dirs_to_process = Vec::new();

    // case where no images have been processed
    if latest_patient_str.is_none() {
        tracing::info!("No patients in database, collecting all directories");

        for entry in dirs {
            let entry = entry.map_err(AppError::from)?;
            let path = entry.path();

            if path.is_dir() {
                dirs_to_process.push(path);
            }
        }

        return Ok(dirs_to_process);
    }

    // case where some images have been processed, and we need to figure out which patients are *new*
    for dir in dirs {
        let dir = dir.map_err(AppError::from)?;
        let path = dir.path();

        if path.is_dir() {
            let dir_name = path.file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("");

            if latest_patient_str.as_ref().map_or(true, |latest| *dir_name > **latest) {
                tracing::info!("Found newer patient directory: {}", dir_name);
                dirs_to_process.push(path);
            }
        }
    }

    Ok(dirs_to_process)
}

pub async fn process_patient_dirs(
    State(pool): State<PgPool>,
    base_url: &str
) -> Result<(), AppError> {
    let patient_dirs = get_patient_dirs_to_process(&pool, base_url).await?;

    for patient_dir in patient_dirs {
        // get last part of the directory full path
        let file_name = patient_dir.file_name().unwrap_or_default();
        let patient_nb = file_name.to_string_lossy();

        let entries = fs::read_dir(&patient_dir)
            .map_err(|e| {
                eprintln!("Error reading dir {}: {}", patient_nb, e);
                e
            })?;

        for file in entries {
            let file = file.map_err(|e| {
                eprintln!("Error with entry in {}: {}", patient_nb, e);
                e
            })?;

            let entry_name = file.file_name();
            let filename = entry_name.to_string_lossy();

            if filename.starts_with("video_") {
                process_image(&pool, &patient_nb, &filename).await?;
            } else if let Some(mask_type) = determine_mask_type(&filename) {
                process_mask(&pool, &patient_nb, &filename, mask_type).await?;
            }
        }
    }

    Ok(())
}

fn determine_mask_type(filename: &str) -> Option<MaskType> {
    if filename.contains("occlusion_colored_") {
        Some(MaskType::Occlusion)
    } else if filename.contains("saliency_colored_") {
        Some(MaskType::Saliency)
    } else if filename.contains("layer_gradcam_colored_") {
        Some(MaskType::LayerGradcam)
    } else if filename.contains("integrated_gradients_colored_") {
        Some(MaskType::IntegratedGradients)
    } else if filename.contains("guided_gradcam_colored_") {
        Some(MaskType::GuidedGradcam)
    } else if filename.contains("gradient_shap_colored_") {
        Some(MaskType::GradientShap)
    } else {
        None
    }
}

async fn process_image(pool: &PgPool, patient_nb: &str, filename: &str) -> Result<i32, AppError> {
    let existing_image = sqlx::query!(
        "SELECT id FROM images WHERE patient_nb = $1 AND filename = $2",
        patient_nb,
        filename
    )
        .fetch_optional(pool)
        .await
        .map_err(AppError::from)?;

    if let Some(image) = existing_image {
        println!("Image already exists with id: {}", image.id);
        return Ok(image.id);
    }

    let image_id = sqlx::query!(
        "INSERT INTO images (filename, patient_nb) VALUES ($1, $2) RETURNING id",
        filename,
        patient_nb
    )
        .fetch_one(pool)
        .await
        .map_err(AppError::from)?
        .id;

    println!("Added new image with id: {}", image_id);

    Ok(image_id)
}

async fn process_mask(pool: &PgPool, patient_nb: &str, filename: &str, mask_type: MaskType) -> Result<i32, AppError> {
    println!("Processing mask: {} of type: {:?}", filename, mask_type);

    let video_prefix = "video_";
    // e.g.: saliency_colored_video_0000_1383.jpg → video_0000_1383.jpg
    let image_filename = if let Some(video_pos) = filename.find(video_prefix) {
        // Extract image_name from filename (i.e. mask name) after video_prefix until the end of the file name
        format!("{}{}", video_prefix, &filename[video_pos + video_prefix.len()..])
    } else {
        return Err(AppError::from(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Could not determine original image for mask: {}", filename),
        )));
    };
    
    // Get image
    let image = sqlx::query!(
        "SELECT id FROM images WHERE patient_nb = $1 AND filename = $2",
        patient_nb,
        image_filename
    )
        .fetch_optional(pool)
        .await
        .map_err(AppError::from)?;

    // Create Image if it doesn't exist
    // This would happen if you go through a directory at random, and the mask is processed before the image
    let image_id = match image {
        Some(img) => img.id,
        None => {
            println!("Original image not found, creating it: {}", image_filename);
            process_image(pool, patient_nb, &image_filename).await?
        }
    };

    // Check if the mask already exists
    // This would happen if there is an interruption while processing a patient's dir
    let existing_mask = sqlx::query!(
        "SELECT id FROM masks WHERE image_id = $1 AND mask_type = $2 AND filename = $3",
        image_id,
        mask_type as _,
        filename
    )
        .fetch_optional(pool)
        .await
        .map_err(AppError::from)?;

    if let Some(mask) = existing_mask {
        println!("Mask already exists with id: {}", mask.id);
        return Ok(mask.id);
    }

    let mask_id = sqlx::query!(
        "INSERT INTO masks (image_id, mask_type, filename) VALUES ($1, $2, $3) RETURNING id",
        image_id,
        mask_type as _,
        filename
    )
        .fetch_one(pool)
        .await
        .map_err(AppError::from)?
        .id;

    println!("Added new mask with id: {}", mask_id);

    Ok(mask_id)
}
