use sqlx::PgPool;
use uuid::Uuid;

use crate::dtos::experience::{CreateExperienceRequest, UpdateExperienceRequest};
use crate::error::{AppError, AppResult};
use crate::models::experience::Experience;
use crate::repositories::experience_repository;

pub async fn list(pool: &PgPool, user_id: Uuid) -> AppResult<Vec<Experience>> {
    experience_repository::list_by_user_id(pool, user_id).await
}

pub async fn create(
    pool: &PgPool,
    user_id: Uuid,
    request: CreateExperienceRequest,
) -> AppResult<Experience> {
    // Experiências pertencem a um perfil, então o usuário precisa ter criado o dele antes.
    experience_repository::create(pool, user_id, &request)
        .await?
        .ok_or(AppError::NotFound(
            "perfil (crie seu perfil antes de adicionar experiências)",
        ))
}

pub async fn update(
    pool: &PgPool,
    user_id: Uuid,
    experience_id: Uuid,
    request: UpdateExperienceRequest,
) -> AppResult<Experience> {
    experience_repository::update(pool, user_id, experience_id, &request)
        .await?
        .ok_or(AppError::NotFound("experiência"))
}

pub async fn delete(pool: &PgPool, user_id: Uuid, experience_id: Uuid) -> AppResult<()> {
    if experience_repository::delete(pool, user_id, experience_id).await? {
        Ok(())
    } else {
        Err(AppError::NotFound("experiência"))
    }
}
