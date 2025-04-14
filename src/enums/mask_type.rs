use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, sqlx::Type, ToSchema, Debug, Clone, PartialEq)]
#[sqlx(type_name = "mask_type", rename_all = "snake_case")]
pub enum MaskType {
    Occlusion,
    Saliency,
    LayerGradcam,
    IntegratedGradients,
    GuidedGradcam,
    GradientShap,
}
