use sqlx::PgPool;
use uuid::Uuid;

use crate::dtos::social_link::{CreateSocialLinkRequest, UpdateSocialLinkRequest};
use crate::error::{AppError, AppResult};
use crate::models::social_link::SocialLink;
use crate::repositories::social_link_repository;

pub async fn list(pool: &PgPool, user_id: Uuid) -> AppResult<Vec<SocialLink>> {
    social_link_repository::list_by_user_id(pool, user_id).await
}

pub async fn create(
    pool: &PgPool,
    user_id: Uuid,
    request: CreateSocialLinkRequest,
) -> AppResult<SocialLink> {
    social_link_repository::create(pool, user_id, &request)
        .await?
        .ok_or(AppError::NotFound(
            "perfil (crie seu perfil antes de adicionar links sociais)",
        ))
}

pub async fn update(
    pool: &PgPool,
    user_id: Uuid,
    social_link_id: Uuid,
    request: UpdateSocialLinkRequest,
) -> AppResult<SocialLink> {
    social_link_repository::update(pool, user_id, social_link_id, &request)
        .await?
        .ok_or(AppError::NotFound("link social"))
}

pub async fn delete(pool: &PgPool, user_id: Uuid, social_link_id: Uuid) -> AppResult<()> {
    if social_link_repository::delete(pool, user_id, social_link_id).await? {
        Ok(())
    } else {
        Err(AppError::NotFound("link social"))
    }
}
