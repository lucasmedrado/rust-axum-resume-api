use sqlx::PgPool;
use uuid::Uuid;

use crate::dtos::academic_education::{
    CreateAcademicEducationRequest, UpdateAcademicEducationRequest,
};
use crate::error::{AppError, AppResult};
use crate::models::academic_education::AcademicEducation;
use crate::repositories::academic_education_repository;

pub async fn list(pool: &PgPool, user_id: Uuid) -> AppResult<Vec<AcademicEducation>> {
    academic_education_repository::list_by_user_id(pool, user_id).await
}

pub async fn create(
    pool: &PgPool,
    user_id: Uuid,
    request: CreateAcademicEducationRequest,
) -> AppResult<AcademicEducation> {
    academic_education_repository::create(pool, user_id, &request)
        .await?
        .ok_or(AppError::NotFound(
            "perfil (crie seu perfil antes de adicionar formações acadêmicas)",
        ))
}

pub async fn update(
    pool: &PgPool,
    user_id: Uuid,
    education_id: Uuid,
    request: UpdateAcademicEducationRequest,
) -> AppResult<AcademicEducation> {
    academic_education_repository::update(pool, user_id, education_id, &request)
        .await?
        .ok_or(AppError::NotFound("formação acadêmica"))
}

pub async fn delete(pool: &PgPool, user_id: Uuid, education_id: Uuid) -> AppResult<()> {
    if academic_education_repository::delete(pool, user_id, education_id).await? {
        Ok(())
    } else {
        Err(AppError::NotFound("formação acadêmica"))
    }
}
