use axum::extract::FromRequestParts;
use axum::http::header::AUTHORIZATION;
use axum::http::request::Parts;
use uuid::Uuid;

use super::jwt::verify_token;
use crate::error::AppError;
use crate::state::AppState;

/// Usuário autenticado. Basta declará-lo como parâmetro do handler para
/// exigir um header `Authorization: Bearer <token>` válido.
pub struct AuthUser {
    pub id: Uuid,
}

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let header = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .ok_or(AppError::InvalidToken)?;

        let token = header
            .strip_prefix("Bearer ")
            .ok_or(AppError::InvalidToken)?;

        let claims = verify_token(token, &state.config.jwt_secret)?;

        Ok(Self { id: claims.sub })
    }
}
