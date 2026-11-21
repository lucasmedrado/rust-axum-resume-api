use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use uuid::Uuid;

use crate::auth::extractor::AuthUser;
use crate::dtos::social_link::{
    CreateSocialLinkRequest, SocialLinkResponse, UpdateSocialLinkRequest,
};
use crate::error::AppResult;
use crate::extractors::ValidatedJson;
use crate::services::social_link_service;
use crate::state::AppState;

pub async fn list(
    auth: AuthUser,
    State(state): State<AppState>,
) -> AppResult<Json<Vec<SocialLinkResponse>>> {
    let links = social_link_service::list(&state.pool, auth.id).await?;

    Ok(Json(links.into_iter().map(Into::into).collect()))
}

pub async fn create(
    auth: AuthUser,
    State(state): State<AppState>,
    ValidatedJson(payload): ValidatedJson<CreateSocialLinkRequest>,
) -> AppResult<(StatusCode, Json<SocialLinkResponse>)> {
    let link = social_link_service::create(&state.pool, auth.id, payload).await?;

    Ok((StatusCode::CREATED, Json(link.into())))
}

pub async fn update(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(social_link_id): Path<Uuid>,
    ValidatedJson(payload): ValidatedJson<UpdateSocialLinkRequest>,
) -> AppResult<Json<SocialLinkResponse>> {
    let link =
        social_link_service::update(&state.pool, auth.id, social_link_id, payload).await?;

    Ok(Json(link.into()))
}

pub async fn delete(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(social_link_id): Path<Uuid>,
) -> AppResult<StatusCode> {
    social_link_service::delete(&state.pool, auth.id, social_link_id).await?;

    Ok(StatusCode::NO_CONTENT)
}
