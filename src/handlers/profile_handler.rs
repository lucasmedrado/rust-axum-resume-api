use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;

use crate::auth::extractor::AuthUser;
use crate::dtos::profile::{CreateProfileRequest, ProfileResponse, UpdateProfileRequest};
use crate::error::AppResult;
use crate::extractors::ValidatedJson;
use crate::services::profile_service;
use crate::state::AppState;

pub async fn create(
    auth: AuthUser,
    State(state): State<AppState>,
    ValidatedJson(payload): ValidatedJson<CreateProfileRequest>,
) -> AppResult<(StatusCode, Json<ProfileResponse>)> {
    let profile = profile_service::create(&state.pool, auth.id, payload).await?;

    Ok((StatusCode::CREATED, Json(profile.into())))
}

pub async fn get_mine(
    auth: AuthUser,
    State(state): State<AppState>,
) -> AppResult<Json<ProfileResponse>> {
    let profile = profile_service::get(&state.pool, auth.id).await?;

    Ok(Json(profile.into()))
}

pub async fn update(
    auth: AuthUser,
    State(state): State<AppState>,
    ValidatedJson(payload): ValidatedJson<UpdateProfileRequest>,
) -> AppResult<Json<ProfileResponse>> {
    let profile = profile_service::update(&state.pool, auth.id, payload).await?;

    Ok(Json(profile.into()))
}

pub async fn delete(auth: AuthUser, State(state): State<AppState>) -> AppResult<StatusCode> {
    profile_service::delete(&state.pool, auth.id).await?;

    Ok(StatusCode::NO_CONTENT)
}
