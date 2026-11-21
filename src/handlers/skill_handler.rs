use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use uuid::Uuid;

use crate::auth::extractor::AuthUser;
use crate::dtos::skill::{CreateSkillRequest, SkillResponse, UpdateSkillRequest};
use crate::error::AppResult;
use crate::extractors::ValidatedJson;
use crate::services::skill_service;
use crate::state::AppState;

pub async fn list(
    auth: AuthUser,
    State(state): State<AppState>,
) -> AppResult<Json<Vec<SkillResponse>>> {
    let skills = skill_service::list(&state.pool, auth.id).await?;

    Ok(Json(skills.into_iter().map(Into::into).collect()))
}

pub async fn create(
    auth: AuthUser,
    State(state): State<AppState>,
    ValidatedJson(payload): ValidatedJson<CreateSkillRequest>,
) -> AppResult<(StatusCode, Json<SkillResponse>)> {
    let skill = skill_service::create(&state.pool, auth.id, payload).await?;

    Ok((StatusCode::CREATED, Json(skill.into())))
}

pub async fn update(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(skill_id): Path<Uuid>,
    ValidatedJson(payload): ValidatedJson<UpdateSkillRequest>,
) -> AppResult<Json<SkillResponse>> {
    let skill = skill_service::update(&state.pool, auth.id, skill_id, payload).await?;

    Ok(Json(skill.into()))
}

pub async fn delete(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(skill_id): Path<Uuid>,
) -> AppResult<StatusCode> {
    skill_service::delete(&state.pool, auth.id, skill_id).await?;

    Ok(StatusCode::NO_CONTENT)
}
