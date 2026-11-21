use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use uuid::Uuid;

use crate::auth::extractor::AuthUser;
use crate::dtos::academic_education::{
    AcademicEducationResponse, CreateAcademicEducationRequest, UpdateAcademicEducationRequest,
};
use crate::error::AppResult;
use crate::extractors::ValidatedJson;
use crate::services::academic_education_service;
use crate::state::AppState;

pub async fn list(
    auth: AuthUser,
    State(state): State<AppState>,
) -> AppResult<Json<Vec<AcademicEducationResponse>>> {
    let educations = academic_education_service::list(&state.pool, auth.id).await?;

    Ok(Json(educations.into_iter().map(Into::into).collect()))
}

pub async fn create(
    auth: AuthUser,
    State(state): State<AppState>,
    ValidatedJson(payload): ValidatedJson<CreateAcademicEducationRequest>,
) -> AppResult<(StatusCode, Json<AcademicEducationResponse>)> {
    let education = academic_education_service::create(&state.pool, auth.id, payload).await?;

    Ok((StatusCode::CREATED, Json(education.into())))
}

pub async fn update(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(education_id): Path<Uuid>,
    ValidatedJson(payload): ValidatedJson<UpdateAcademicEducationRequest>,
) -> AppResult<Json<AcademicEducationResponse>> {
    let education =
        academic_education_service::update(&state.pool, auth.id, education_id, payload).await?;

    Ok(Json(education.into()))
}

pub async fn delete(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(education_id): Path<Uuid>,
) -> AppResult<StatusCode> {
    academic_education_service::delete(&state.pool, auth.id, education_id).await?;

    Ok(StatusCode::NO_CONTENT)
}
