use axum::{
    extract::{Path, State}, http::StatusCode,
    routing::{get, post},
    Json,
    Router,
};
use serde_json::to_string_pretty;
use sqlx::PgPool;

use crate::error::AppError;
use crate::models::image::ImageSet;
use crate::models::preference::{
    CreatePreference, Preference, PreferenceEvent, PreferenceWithEvents,
};
use crate::routes::images::get_image_with_masks;

pub fn routes() -> Router<PgPool> {
    Router::new()
        .route("/preferences", post(create_preference).get(get_preferences))
        .route("/preferences/{id}", get(get_preference))
        .route("/users/{user_id}/next-image", get(get_next_image))
}

#[utoipa::path(
    post,
    path = "/preferences",
    request_body = CreatePreference,
    responses(
        (status = 201, description = "Created preference with success", body = PreferenceWithEvents),
        (status = 500, description = "Internal Server Error")
    ),
    tag = "preferences"
)]
pub async fn create_preference(
    State(pool): State<PgPool>,
    Json(payload): Json<CreatePreference>,
) -> Result<(StatusCode, Json<PreferenceWithEvents>), AppError> {
    let mut tx = pool.begin().await.map_err(AppError::from)?;

    let user_exists = sqlx::query!(
        "SELECT EXISTS(SELECT 1 FROM users WHERE id = $1) as exists",
        payload.user_id
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(AppError::from)?
    .exists
    .unwrap_or(false);

    if !user_exists {
        return Err(AppError::NotFound);
    }

    let image_exists = sqlx::query!(
        "SELECT EXISTS(SELECT 1 FROM images WHERE id = $1) as exists",
        payload.image_id
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(AppError::from)?
    .exists
    .unwrap_or(false);

    if !image_exists {
        return Err(AppError::NotFound);
    }

    let existing_preference = sqlx::query!(
        "SELECT id FROM preferences WHERE user_id = $1 AND image_id = $2",
        payload.user_id,
        payload.image_id
    )
    .fetch_optional(&mut *tx)
    .await
    .map_err(AppError::from)?;

    if existing_preference.is_some() {
        sqlx::query!(
            "DELETE FROM preference_events WHERE preference_id = $1",
            existing_preference.as_ref().unwrap().id
        )
        .execute(&mut *tx)
        .await
        .map_err(AppError::from)?;

        sqlx::query!(
            "DELETE FROM preferences WHERE id = $1",
            existing_preference.as_ref().unwrap().id
        )
        .execute(&mut *tx)
        .await
        .map_err(AppError::from)?;
    }

    let preference = sqlx::query_as!(
        Preference,
        "INSERT INTO preferences (user_id, image_id) VALUES ($1, $2) RETURNING id, user_id, image_id, created_at",
        payload.user_id,
        payload.image_id
    )
        .fetch_one(&mut *tx)
        .await
        .map_err(AppError::from)?;

    let preference_id = preference.id;

    for event in &payload.events {
        let mask_valid = sqlx::query!(
            "SELECT EXISTS(SELECT 1 FROM masks WHERE id = $1 AND image_id = $2) as exists",
            event.mask_id,
            payload.image_id
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(AppError::from)?
        .exists
        .unwrap_or(false);

        if !mask_valid {
            return Err(AppError::BadRequest(format!(
                "Masque {} non valide pour l'image {}",
                event.mask_id, payload.image_id
            )));
        }
    }

    let mut events = Vec::new();
    for event in payload.events {
        let preference_event = sqlx::query_as!(
            PreferenceEvent,
            "INSERT INTO preference_events (preference_id, mask_id, rank) VALUES ($1, $2, $3) RETURNING id, preference_id, mask_id, rank",
            preference_id,
            event.mask_id,
            event.rank
        )
            .fetch_one(&mut *tx)
            .await
            .map_err(AppError::from)?;

        events.push(preference_event);
    }

    tx.commit().await.map_err(AppError::from)?;

    let preference_with_events = PreferenceWithEvents { preference, events };

    tracing::info!(
        "Preference created: {}",
        to_string_pretty(&preference_with_events).unwrap()
    );

    Ok((StatusCode::CREATED, Json(preference_with_events)))
}

#[utoipa::path(
    get,
    path = "/preferences",
    responses(
        (status = 200, description = "List all preferences", body = Vec<PreferenceWithEvents>),
        (status = 500, description = "Internal Server Error")
    ),
    tag = "preferences"
)]
pub async fn get_preferences(
    State(pool): State<PgPool>,
) -> Result<Json<Vec<PreferenceWithEvents>>, AppError> {
    let preferences = sqlx::query_as!(
        Preference,
        "SELECT id, user_id, image_id, created_at FROM preferences ORDER BY created_at DESC"
    )
    .fetch_all(&pool)
    .await
    .map_err(AppError::from)?;

    let mut preferences_with_events = Vec::new();
    for preference in preferences {
        let events = sqlx::query_as!(
            PreferenceEvent,
            "SELECT id, preference_id, mask_id, rank FROM preference_events WHERE preference_id = $1 ORDER BY rank",
            preference.id
        )
            .fetch_all(&pool)
            .await
            .map_err(AppError::from)?;

        preferences_with_events.push(PreferenceWithEvents { preference, events });
    }

    tracing::info!("Found preferences: {}", preferences_with_events.len());

    Ok(Json(preferences_with_events))
}

#[utoipa::path(
    get,
    path = "/preferences/{id}",
    responses(
        (status = 200, description = "Found preference", body = PreferenceWithEvents),
        (status = 404, description = "Preference not found"),
        (status = 500, description = "Internal Server Error")
    ),
    tag = "preferences"
)]
pub async fn get_preference(
    State(pool): State<PgPool>,
    Path(id): Path<i32>,
) -> Result<Json<PreferenceWithEvents>, AppError> {
    let preference = sqlx::query_as!(
        Preference,
        "SELECT id, user_id, image_id, created_at FROM preferences WHERE id = $1",
        id
    )
    .fetch_optional(&pool)
    .await
    .map_err(AppError::from)?
    .ok_or_else(|| AppError::NotFound)?;

    let events = sqlx::query_as!(
        PreferenceEvent,
        "SELECT id, preference_id, mask_id, rank FROM preference_events WHERE preference_id = $1 ORDER BY rank",
        preference.id
    )
        .fetch_all(&pool)
        .await
        .map_err(AppError::from)?;

    let preference_with_events = PreferenceWithEvents { preference, events };

    tracing::info!(
        "Found preference: {}",
        to_string_pretty(&preference_with_events).unwrap()
    );

    Ok(Json(preference_with_events))
}

// Todo needs full refactoring -> not the right feature. Need to look at the # of patient from Image.patient_nb
#[utoipa::path(
    get,
    path = "/users/{user_id}/next-image",
    responses(
        (status = 200, description = "Next Image Sequence Found", body = ImageSet),
        (status = 404, description = "No further Image Sequence is available"),
        (status = 500, description = "Erreur interne du serveur")
    ),
    tag = "images"
)]
pub async fn get_next_image(
    State(pool): State<PgPool>,
    Path(user_id): Path<i32>,
) -> Result<Json<ImageSet>, AppError> {
    let user_exists = sqlx::query!(
        "SELECT EXISTS(SELECT 1 FROM users WHERE id = $1) as exists",
        user_id
    )
    .fetch_one(&pool)
    .await
    .map_err(AppError::from)?
    .exists
    .unwrap_or(false);

    if !user_exists {
        return Err(AppError::NotFound);
    }

    let next_image = sqlx::query!(
        r#"
        SELECT i.id FROM images i
        WHERE NOT EXISTS (
            SELECT 1 FROM preferences p
            WHERE p.image_id = i.id AND p.user_id = $1
        )
        ORDER BY i.id
        LIMIT 1
        "#,
        user_id
    )
    .fetch_optional(&pool)
    .await
    .map_err(AppError::from)?;

    match next_image {
        Some(image) => get_image_with_masks(State(pool), Path(image.id)).await,
        None => Err(AppError::NotFound),
    }
}
