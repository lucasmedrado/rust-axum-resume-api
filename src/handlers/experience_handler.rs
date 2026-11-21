use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use uuid::Uuid;

use crate::auth::extractor::AuthUser;
use crate::dtos::experience::{
    CreateExperienceRequest, ExperienceResponse, UpdateExperienceRequest,
};
use crate::error::AppResult;
use crate::extractors::ValidatedJson;
use crate::services::experience_service;
use crate::state::AppState;

pub async fn list(
    auth: AuthUser,
    State(state): State<AppState>,
) -> AppResult<Json<Vec<ExperienceResponse>>> {
    let experiences = experience_service::list(&state.pool, auth.id).await?;

    Ok(Json(experiences.into_iter().map(Into::into).collect()))
}

pub async fn create(
    auth: AuthUser,
    State(state): State<AppState>,
    ValidatedJson(payload): ValidatedJson<CreateExperienceRequest>,
) -> AppResult<(StatusCode, Json<ExperienceResponse>)> {
    let experience = experience_service::create(&state.pool, auth.id, payload).await?;

    Ok((StatusCode::CREATED, Json(experience.into())))
}

pub async fn update(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(experience_id): Path<Uuid>,
    ValidatedJson(payload): ValidatedJson<UpdateExperienceRequest>,
) -> AppResult<Json<ExperienceResponse>> {
    let experience =
        experience_service::update(&state.pool, auth.id, experience_id, payload).await?;

    Ok(Json(experience.into()))
}

pub async fn delete(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(experience_id): Path<Uuid>,
) -> AppResult<StatusCode> {
    experience_service::delete(&state.pool, auth.id, experience_id).await?;

    Ok(StatusCode::NO_CONTENT)
}
