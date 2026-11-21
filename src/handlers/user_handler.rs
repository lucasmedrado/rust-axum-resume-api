use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;

use crate::auth::extractor::AuthUser;
use crate::dtos::user::{
    AuthResponse, LoginRequest, RegisterRequest, TokenResponse, UpdateUserRequest, UserResponse,
};
use crate::error::AppResult;
use crate::extractors::ValidatedJson;
use crate::services::user_service;
use crate::state::AppState;

pub async fn register(
    State(state): State<AppState>,
    ValidatedJson(payload): ValidatedJson<RegisterRequest>,
) -> AppResult<(StatusCode, Json<AuthResponse>)> {
    let (user, token) = user_service::register(&state, payload).await?;

    let response = AuthResponse {
        user: user.into(),
        token,
    };

    Ok((StatusCode::CREATED, Json(response)))
}

pub async fn login(
    State(state): State<AppState>,
    ValidatedJson(payload): ValidatedJson<LoginRequest>,
) -> AppResult<Json<TokenResponse>> {
    let token = user_service::login(&state, payload).await?;

    Ok(Json(TokenResponse { token }))
}

pub async fn get_me(
    auth: AuthUser,
    State(state): State<AppState>,
) -> AppResult<Json<UserResponse>> {
    let user = user_service::get(&state.pool, auth.id).await?;

    Ok(Json(user.into()))
}

pub async fn update_me(
    auth: AuthUser,
    State(state): State<AppState>,
    ValidatedJson(payload): ValidatedJson<UpdateUserRequest>,
) -> AppResult<Json<UserResponse>> {
    let user = user_service::update(&state.pool, auth.id, payload).await?;

    Ok(Json(user.into()))
}

pub async fn delete_me(auth: AuthUser, State(state): State<AppState>) -> AppResult<StatusCode> {
    user_service::delete(&state.pool, auth.id).await?;

    Ok(StatusCode::NO_CONTENT)
}
