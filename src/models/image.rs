use crate::models::mask::MaskImage;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, sqlx::FromRow, ToSchema, Debug, Clone)]
pub struct Image {
    /// Unique id for the image
    pub id: i32,
    /// Filename
    pub filename: String,
    /// Patient number i.e. containing folder
    pub patient_nb: String,
}

#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
pub struct ImageData {
    #[serde(flatten)]
    pub image: Image,

    /// image content type (e.g. "image/jpeg")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,
    /// base64 data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct ImageSet {
    /// URL of the image
    pub original_image: String,
    /// masks linked to the image
    pub masks: Vec<MaskImage>,
}

#[derive(Serialize, ToSchema)]
pub struct ImageDataSet {
    /// image
    pub image: ImageData,
    /// masks linked to the image
    pub masks: Vec<MaskImage>,
}
