use crate::enums::mask_type::MaskType;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, sqlx::FromRow, ToSchema, Debug, Clone)]
pub struct Mask {
    /// id
    pub id: i32,
    /// id of the linked image
    pub image_id: i32,
    /// mask type
    pub mask_type: MaskType,
    /// mask file name
    pub filename: String,
}

#[derive(Serialize, ToSchema)]
pub struct MaskImage {
    /// id
    pub id: String,
    /// mask type
    pub r#type: String,
    /// url
    pub image_url: String,
    /// image content type (e.g. "image/jpeg")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,
    /// base64 data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_data: Option<String>,
}
