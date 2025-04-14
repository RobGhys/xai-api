use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, sqlx::FromRow, ToSchema, Debug, Clone)]
pub struct Preference {
    pub id: i32,
    pub user_id: i32,
    pub image_id: i32,
    pub created_at: OffsetDateTime,
}

#[derive(Serialize, Deserialize, sqlx::FromRow, ToSchema, Debug, Clone)]
pub struct PreferenceEvent {
    pub id: i32,
    pub preference_id: i32,
    pub mask_id: i32,
    pub rank: i32,
}

#[derive(Deserialize, ToSchema)]
pub struct CreatePreference {
    pub user_id: i32,
    pub image_id: i32,
    pub events: Vec<CreatePreferenceEvent>,
}

#[derive(Deserialize, ToSchema)]
pub struct CreatePreferenceEvent {
    /// id of the mask
    pub mask_id: i32,
    /// rank
    pub rank: i32,
}

#[derive(Serialize, ToSchema)]
pub struct PreferenceWithEvents {
    /// preference
    pub preference: Preference,
    /// events
    pub events: Vec<PreferenceEvent>,
}

#[derive(Serialize, ToSchema)]
pub struct RankedMask {
    /// id
    pub id: String,
    /// mask type
    pub r#type: String,
    /// display
    pub image_url: String,
    /// name
    pub name: String,
    /// rank position
    pub rank: i32,
}
