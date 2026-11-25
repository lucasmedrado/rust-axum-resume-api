use sqlx::PgPool;
use uuid::Uuid;

use crate::dtos::experience::{CreateExperienceRequest, UpdateExperienceRequest};
use crate::error::{AppError, AppResult, is_check_violation};
use crate::models::experience::Experience;


pub async fn list_by_user_id(pool: &PgPool, user_id: Uuid) -> AppResult<Vec<Experience>> {
    let experiences = sqlx::query_as::<_, Experience>(
        "SELECT e.id, e.profile_id, e.company_name, e.position, e.start_date, e.end_date, e.description
         FROM experiences e
         JOIN profiles p ON p.id = e.profile_id
         WHERE p.user_id = $1
         ORDER BY e.start_date DESC, e.id",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    Ok(experiences)
}


pub async fn create(
    pool: &PgPool,
    user_id: Uuid,
    request: &CreateExperienceRequest,
) -> AppResult<Option<Experience>> {
    sqlx::query_as::<_, Experience>(
        "INSERT INTO experiences (profile_id, company_name, position, start_date, end_date, description)
         SELECT p.id, $2, $3, $4, $5, $6
         FROM profiles p
         WHERE p.user_id = $1
         RETURNING id, profile_id, company_name, position, start_date, end_date, description",
    )
    .bind(user_id)
    .bind(&request.company_name)
    .bind(&request.position)
    .bind(request.start_date)
    .bind(request.end_date)
    .bind(request.description.as_deref())
    .fetch_optional(pool)
    .await
    .map_err(map_write_error)
}


pub async fn update(
    pool: &PgPool,
    user_id: Uuid,
    experience_id: Uuid,
    request: &UpdateExperienceRequest,
) -> AppResult<Option<Experience>> {
    sqlx::query_as::<_, Experience>(
        "UPDATE experiences e
         SET company_name = COALESCE($3, e.company_name),
             position = COALESCE($4, e.position),
             start_date = COALESCE($5, e.start_date),
             end_date = COALESCE($6, e.end_date),
             description = COALESCE($7, e.description)
         FROM profiles p
         WHERE e.id = $2 AND e.profile_id = p.id AND p.user_id = $1
         RETURNING e.id, e.profile_id, e.company_name, e.position, e.start_date, e.end_date, e.description",
    )
    .bind(user_id)
    .bind(experience_id)
    .bind(request.company_name.as_deref())
    .bind(request.position.as_deref())
    .bind(request.start_date)
    .bind(request.end_date)
    .bind(request.description.as_deref())
    .fetch_optional(pool)
    .await
    .map_err(map_write_error)
}


pub async fn delete(pool: &PgPool, user_id: Uuid, experience_id: Uuid) -> AppResult<bool> {
    let result = sqlx::query(
        "DELETE FROM experiences e
         USING profiles p
         WHERE e.id = $2 AND e.profile_id = p.id AND p.user_id = $1",
    )
    .bind(user_id)
    .bind(experience_id)
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

fn map_write_error(err: sqlx::Error) -> AppError {
    if is_check_violation(&err) {
        AppError::Unprocessable("A data final não pode ser anterior à data inicial")
    } else {
        AppError::Database(err)
    }
}
