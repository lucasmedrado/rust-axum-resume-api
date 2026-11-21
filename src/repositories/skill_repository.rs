use sqlx::PgPool;
use uuid::Uuid;

use crate::dtos::skill::{CreateSkillRequest, UpdateSkillRequest};
use crate::error::{AppError, AppResult, is_check_violation, unique_violation};
use crate::models::skill::Skill;

// Todas as consultas ligam `skills` a `profiles` para garantir que
// a habilidade pertence ao usuário autenticado.

pub async fn list_by_user_id(pool: &PgPool, user_id: Uuid) -> AppResult<Vec<Skill>> {
    let skills = sqlx::query_as::<_, Skill>(
        "SELECT s.id, s.profile_id, s.name, s.skill_type, s.level
         FROM skills s
         JOIN profiles p ON p.id = s.profile_id
         WHERE p.user_id = $1
         ORDER BY s.skill_type ASC, lower(s.name) ASC, s.id",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    Ok(skills)
}

/// Devolve `None` quando o usuário ainda não tem perfil.
pub async fn create(
    pool: &PgPool,
    user_id: Uuid,
    request: &CreateSkillRequest,
) -> AppResult<Option<Skill>> {
    sqlx::query_as::<_, Skill>(
        "INSERT INTO skills (profile_id, name, skill_type, level)
         SELECT p.id, $2, $3, $4
         FROM profiles p
         WHERE p.user_id = $1
         RETURNING id, profile_id, name, skill_type, level",
    )
    .bind(user_id)
    .bind(&request.name)
    .bind(&request.skill_type)
    .bind(request.level.as_deref())
    .fetch_optional(pool)
    .await
    .map_err(map_write_error)
}

/// Atualiza apenas os campos `Some`. Devolve `None` se a habilidade não existe
/// ou não pertence ao usuário.
pub async fn update(
    pool: &PgPool,
    user_id: Uuid,
    skill_id: Uuid,
    request: &UpdateSkillRequest,
) -> AppResult<Option<Skill>> {
    sqlx::query_as::<_, Skill>(
        "UPDATE skills s
         SET name = COALESCE($3, s.name),
             skill_type = COALESCE($4, s.skill_type),
             level = COALESCE($5, s.level)
         FROM profiles p
         WHERE s.id = $2 AND s.profile_id = p.id AND p.user_id = $1
         RETURNING s.id, s.profile_id, s.name, s.skill_type, s.level",
    )
    .bind(user_id)
    .bind(skill_id)
    .bind(request.name.as_deref())
    .bind(request.skill_type.as_deref())
    .bind(request.level.as_deref())
    .fetch_optional(pool)
    .await
    .map_err(map_write_error)
}

/// Devolve `true` se a habilidade existia, era do usuário e foi removida.
pub async fn delete(pool: &PgPool, user_id: Uuid, skill_id: Uuid) -> AppResult<bool> {
    let result = sqlx::query(
        "DELETE FROM skills s
         USING profiles p
         WHERE s.id = $2 AND s.profile_id = p.id AND p.user_id = $1",
    )
    .bind(user_id)
    .bind(skill_id)
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

fn map_write_error(err: sqlx::Error) -> AppError {
    match unique_violation(&err) {
        Some("skills_profile_name_uidx") => {
            AppError::Conflict("esta habilidade já está cadastrada no seu perfil")
        }
        _ if is_check_violation(&err) => {
            AppError::Unprocessable("Tipo ou nível de habilidade inválido")
        }
        _ => AppError::Database(err),
    }
}
