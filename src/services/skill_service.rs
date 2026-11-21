use sqlx::PgPool;
use uuid::Uuid;

use crate::dtos::skill::{CreateSkillRequest, UpdateSkillRequest};
use crate::error::{AppError, AppResult};
use crate::models::skill::Skill;
use crate::repositories::skill_repository;

pub async fn list(pool: &PgPool, user_id: Uuid) -> AppResult<Vec<Skill>> {
    skill_repository::list_by_user_id(pool, user_id).await
}

pub async fn create(
    pool: &PgPool,
    user_id: Uuid,
    request: CreateSkillRequest,
) -> AppResult<Skill> {
    skill_repository::create(pool, user_id, &request)
        .await?
        .ok_or(AppError::NotFound(
            "perfil (crie seu perfil antes de adicionar habilidades)",
        ))
}

pub async fn update(
    pool: &PgPool,
    user_id: Uuid,
    skill_id: Uuid,
    request: UpdateSkillRequest,
) -> AppResult<Skill> {
    skill_repository::update(pool, user_id, skill_id, &request)
        .await?
        .ok_or(AppError::NotFound("habilidade"))
}

pub async fn delete(pool: &PgPool, user_id: Uuid, skill_id: Uuid) -> AppResult<()> {
    if skill_repository::delete(pool, user_id, skill_id).await? {
        Ok(())
    } else {
        Err(AppError::NotFound("habilidade"))
    }
}
