use sqlx::PgPool;
use uuid::Uuid;

use crate::dtos::certification::{CreateCertificationRequest, UpdateCertificationRequest};
use crate::error::{AppError, AppResult};
use crate::models::certification::Certification;
use crate::repositories::certification_repository;

pub async fn list(pool: &PgPool, user_id: Uuid) -> AppResult<Vec<Certification>> {
    certification_repository::list_by_user_id(pool, user_id).await
}

pub async fn create(
    pool: &PgPool,
    user_id: Uuid,
    request: CreateCertificationRequest,
) -> AppResult<Certification> {
    certification_repository::create(pool, user_id, &request)
        .await?
        .ok_or(AppError::NotFound(
            "perfil (crie seu perfil antes de adicionar certificações)",
        ))
}

pub async fn update(
    pool: &PgPool,
    user_id: Uuid,
    certification_id: Uuid,
    request: UpdateCertificationRequest,
) -> AppResult<Certification> {
    certification_repository::update(pool, user_id, certification_id, &request)
        .await?
        .ok_or(AppError::NotFound("certificação"))
}

pub async fn delete(pool: &PgPool, user_id: Uuid, certification_id: Uuid) -> AppResult<()> {
    if certification_repository::delete(pool, user_id, certification_id).await? {
        Ok(())
    } else {
        Err(AppError::NotFound("certificação"))
    }
}
