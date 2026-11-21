use sqlx::PgPool;
use uuid::Uuid;

use crate::dtos::social_link::{CreateSocialLinkRequest, UpdateSocialLinkRequest};
use crate::error::{AppError, AppResult, is_check_violation, unique_violation};
use crate::models::social_link::SocialLink;

// Todas as consultas ligam `social_links` a `profiles` para garantir que
// o link social pertence ao usuário autenticado.

pub async fn list_by_user_id(pool: &PgPool, user_id: Uuid) -> AppResult<Vec<SocialLink>> {
    let links = sqlx::query_as::<_, SocialLink>(
        "SELECT sl.id, sl.profile_id, sl.platform_name, sl.url
         FROM social_links sl
         JOIN profiles p ON p.id = sl.profile_id
         WHERE p.user_id = $1
         ORDER BY lower(sl.platform_name) ASC, sl.id",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    Ok(links)
}

/// Devolve `None` quando o usuário ainda não tem perfil.
pub async fn create(
    pool: &PgPool,
    user_id: Uuid,
    request: &CreateSocialLinkRequest,
) -> AppResult<Option<SocialLink>> {
    sqlx::query_as::<_, SocialLink>(
        "INSERT INTO social_links (profile_id, platform_name, url)
         SELECT p.id, $2, $3
         FROM profiles p
         WHERE p.user_id = $1
         RETURNING id, profile_id, platform_name, url",
    )
    .bind(user_id)
    .bind(&request.platform_name)
    .bind(&request.url)
    .fetch_optional(pool)
    .await
    .map_err(map_write_error)
}

/// Atualiza apenas os campos `Some`. Devolve `None` se o link social não existe
/// ou não pertence ao usuário.
pub async fn update(
    pool: &PgPool,
    user_id: Uuid,
    social_link_id: Uuid,
    request: &UpdateSocialLinkRequest,
) -> AppResult<Option<SocialLink>> {
    sqlx::query_as::<_, SocialLink>(
        "UPDATE social_links sl
         SET platform_name = COALESCE($3, sl.platform_name),
             url = COALESCE($4, sl.url)
         FROM profiles p
         WHERE sl.id = $2 AND sl.profile_id = p.id AND p.user_id = $1
         RETURNING sl.id, sl.profile_id, sl.platform_name, sl.url",
    )
    .bind(user_id)
    .bind(social_link_id)
    .bind(request.platform_name.as_deref())
    .bind(request.url.as_deref())
    .fetch_optional(pool)
    .await
    .map_err(map_write_error)
}

/// Devolve `true` se o link social existia, era do usuário e foi removido.
pub async fn delete(pool: &PgPool, user_id: Uuid, social_link_id: Uuid) -> AppResult<bool> {
    let result = sqlx::query(
        "DELETE FROM social_links sl
         USING profiles p
         WHERE sl.id = $2 AND sl.profile_id = p.id AND p.user_id = $1",
    )
    .bind(user_id)
    .bind(social_link_id)
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

fn map_write_error(err: sqlx::Error) -> AppError {
    match unique_violation(&err) {
        Some("social_links_profile_platform_uidx") => {
            AppError::Conflict("esta plataforma já está cadastrada no seu perfil")
        }
        _ if is_check_violation(&err) => {
            AppError::Unprocessable("Dados do link social violam as regras de formato")
        }
        _ => AppError::Database(err),
    }
}
