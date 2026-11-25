use sqlx::PgPool;
use uuid::Uuid;

use crate::dtos::profile::{CreateProfileRequest, UpdateProfileRequest};
use crate::error::{AppError, AppResult, is_foreign_key_violation, unique_violation};
use crate::models::profile::Profile;

pub async fn create(
    pool: &PgPool,
    user_id: Uuid,
    request: &CreateProfileRequest,
) -> AppResult<Profile> {
    sqlx::query_as::<_, Profile>(
        "INSERT INTO profiles (user_id, full_name, localization, professional_objective, slug, bio)
         VALUES ($1, $2, $3, $4, $5, $6)
         RETURNING id, user_id, full_name, localization, professional_objective, slug, bio, is_public",
    )
    .bind(user_id)
    .bind(&request.full_name)
    .bind(&request.localization)
    .bind(request.professional_objective.as_deref())
    .bind(&request.slug)
    .bind(request.bio.as_deref())
    .fetch_one(pool)
    .await
    .map_err(map_write_error)
}

pub async fn find_by_user_id(pool: &PgPool, user_id: Uuid) -> AppResult<Option<Profile>> {
    let profile = sqlx::query_as::<_, Profile>(
        "SELECT id, user_id, full_name, localization, professional_objective, slug, bio, is_public
         FROM profiles
         WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    Ok(profile)
}


pub async fn update(
    pool: &PgPool,
    user_id: Uuid,
    request: &UpdateProfileRequest,
) -> AppResult<Option<Profile>> {
    sqlx::query_as::<_, Profile>(
        "UPDATE profiles
         SET full_name = COALESCE($2, full_name),
             localization = COALESCE($3, localization),
             professional_objective = COALESCE($4, professional_objective),
             slug = COALESCE($5, slug),
             bio = COALESCE($6, bio),
             is_public = COALESCE($7, is_public)
         WHERE user_id = $1
         RETURNING id, user_id, full_name, localization, professional_objective, slug, bio, is_public",
    )
    .bind(user_id)
    .bind(request.full_name.as_deref())
    .bind(request.localization.as_deref())
    .bind(request.professional_objective.as_deref())
    .bind(request.slug.as_deref())
    .bind(request.bio.as_deref())
    .bind(request.is_public)
    .fetch_optional(pool)
    .await
    .map_err(map_write_error)
}


pub async fn delete(pool: &PgPool, user_id: Uuid) -> AppResult<bool> {
    let result = sqlx::query("DELETE FROM profiles WHERE user_id = $1")
        .bind(user_id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected() > 0)
}

fn map_write_error(err: sqlx::Error) -> AppError {
    match unique_violation(&err) {
        Some("profiles_slug_key") => AppError::Conflict("este slug já está em uso"),
        Some("profiles_user_id_key") => AppError::Conflict("o usuário já possui um perfil"),
        _ if is_foreign_key_violation(&err) => AppError::NotFound("usuário"),
        _ => AppError::Database(err),
    }
}
