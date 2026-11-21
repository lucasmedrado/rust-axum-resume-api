use sqlx::PgPool;
use uuid::Uuid;

use crate::dtos::certification::{CreateCertificationRequest, UpdateCertificationRequest};
use crate::error::{AppError, AppResult, is_check_violation};
use crate::models::certification::Certification;

// Todas as consultas ligam `certifications` a `profiles` para garantir que
// a certificação pertence ao usuário autenticado.

pub async fn list_by_user_id(pool: &PgPool, user_id: Uuid) -> AppResult<Vec<Certification>> {
    let certifications = sqlx::query_as::<_, Certification>(
        "SELECT c.id, c.profile_id, c.name, c.issuing_organization, c.issue_date, c.url
         FROM certifications c
         JOIN profiles p ON p.id = c.profile_id
         WHERE p.user_id = $1
         ORDER BY c.issue_date DESC NULLS LAST, c.id",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    Ok(certifications)
}

/// Devolve `None` quando o usuário ainda não tem perfil.
pub async fn create(
    pool: &PgPool,
    user_id: Uuid,
    request: &CreateCertificationRequest,
) -> AppResult<Option<Certification>> {
    sqlx::query_as::<_, Certification>(
        "INSERT INTO certifications (profile_id, name, issuing_organization, issue_date, url)
         SELECT p.id, $2, $3, $4, $5
         FROM profiles p
         WHERE p.user_id = $1
         RETURNING id, profile_id, name, issuing_organization, issue_date, url",
    )
    .bind(user_id)
    .bind(&request.name)
    .bind(request.issuing_organization.as_deref())
    .bind(request.issue_date)
    .bind(request.url.as_deref())
    .fetch_optional(pool)
    .await
    .map_err(map_write_error)
}

/// Atualiza apenas os campos `Some`. Devolve `None` se a certificação não existe
/// ou não pertence ao usuário.
pub async fn update(
    pool: &PgPool,
    user_id: Uuid,
    certification_id: Uuid,
    request: &UpdateCertificationRequest,
) -> AppResult<Option<Certification>> {
    sqlx::query_as::<_, Certification>(
        "UPDATE certifications c
         SET name = COALESCE($3, c.name),
             issuing_organization = COALESCE($4, c.issuing_organization),
             issue_date = COALESCE($5, c.issue_date),
             url = COALESCE($6, c.url)
         FROM profiles p
         WHERE c.id = $2 AND c.profile_id = p.id AND p.user_id = $1
         RETURNING c.id, c.profile_id, c.name, c.issuing_organization, c.issue_date, c.url",
    )
    .bind(user_id)
    .bind(certification_id)
    .bind(request.name.as_deref())
    .bind(request.issuing_organization.as_deref())
    .bind(request.issue_date)
    .bind(request.url.as_deref())
    .fetch_optional(pool)
    .await
    .map_err(map_write_error)
}

/// Devolve `true` se a certificação existia, era do usuário e foi removida.
pub async fn delete(pool: &PgPool, user_id: Uuid, certification_id: Uuid) -> AppResult<bool> {
    let result = sqlx::query(
        "DELETE FROM certifications c
         USING profiles p
         WHERE c.id = $2 AND c.profile_id = p.id AND p.user_id = $1",
    )
    .bind(user_id)
    .bind(certification_id)
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

fn map_write_error(err: sqlx::Error) -> AppError {
    if is_check_violation(&err) {
        AppError::Unprocessable("Dados da certificação violam as regras de formato")
    } else {
        AppError::Database(err)
    }
}
