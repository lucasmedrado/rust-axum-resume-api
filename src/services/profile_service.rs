use sqlx::PgPool;
use uuid::Uuid;

use crate::dtos::profile::{CreateProfileRequest, UpdateProfileRequest};
use crate::error::{AppError, AppResult};
use crate::models::profile::Profile;
use crate::repositories::profile_repository;

pub async fn create(
    pool: &PgPool,
    user_id: Uuid,
    request: CreateProfileRequest,
) -> AppResult<Profile> {
    profile_repository::create(pool, user_id, &request).await
}

pub async fn get(pool: &PgPool, user_id: Uuid) -> AppResult<Profile> {
    profile_repository::find_by_user_id(pool, user_id)
        .await?
        .ok_or(AppError::NotFound("perfil"))
}

pub async fn update(
    pool: &PgPool,
    user_id: Uuid,
    request: UpdateProfileRequest,
) -> AppResult<Profile> {
    profile_repository::update(pool, user_id, &request)
        .await?
        .ok_or(AppError::NotFound("perfil"))
}

pub async fn delete(pool: &PgPool, user_id: Uuid) -> AppResult<()> {
    if profile_repository::delete(pool, user_id).await? {
        Ok(())
    } else {
        Err(AppError::NotFound("perfil"))
    }
}
