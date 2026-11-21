use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use uuid::Uuid;

use crate::auth::extractor::AuthUser;
use crate::dtos::certification::{
    CertificationResponse, CreateCertificationRequest, UpdateCertificationRequest,
};
use crate::error::AppResult;
use crate::extractors::ValidatedJson;
use crate::services::certification_service;
use crate::state::AppState;

pub async fn list(
    auth: AuthUser,
    State(state): State<AppState>,
) -> AppResult<Json<Vec<CertificationResponse>>> {
    let certifications = certification_service::list(&state.pool, auth.id).await?;

    Ok(Json(certifications.into_iter().map(Into::into).collect()))
}

pub async fn create(
    auth: AuthUser,
    State(state): State<AppState>,
    ValidatedJson(payload): ValidatedJson<CreateCertificationRequest>,
) -> AppResult<(StatusCode, Json<CertificationResponse>)> {
    let certification = certification_service::create(&state.pool, auth.id, payload).await?;

    Ok((StatusCode::CREATED, Json(certification.into())))
}

pub async fn update(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(certification_id): Path<Uuid>,
    ValidatedJson(payload): ValidatedJson<UpdateCertificationRequest>,
) -> AppResult<Json<CertificationResponse>> {
    let certification =
        certification_service::update(&state.pool, auth.id, certification_id, payload).await?;

    Ok(Json(certification.into()))
}

pub async fn delete(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(certification_id): Path<Uuid>,
) -> AppResult<StatusCode> {
    certification_service::delete(&state.pool, auth.id, certification_id).await?;

    Ok(StatusCode::NO_CONTENT)
}
