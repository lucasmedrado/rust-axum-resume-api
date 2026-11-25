use sqlx::PgPool;
use uuid::Uuid;

use crate::error::{AppError, AppResult, unique_violation};
use crate::models::user::User;

pub async fn create(pool: &PgPool, email: &str, password_hash: &str) -> AppResult<User> {
    sqlx::query_as::<_, User>(
        "INSERT INTO users (email, password_hash)
         VALUES ($1, $2)
         RETURNING id, email, password_hash, created_at",
    )
    .bind(email)
    .bind(password_hash)
    .fetch_one(pool)
    .await
    .map_err(map_write_error)
}

pub async fn find_by_id(pool: &PgPool, id: Uuid) -> AppResult<Option<User>> {
    let user = sqlx::query_as::<_, User>(
        "SELECT id, email, password_hash, created_at FROM users WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    Ok(user)
}

pub async fn find_by_email(pool: &PgPool, email: &str) -> AppResult<Option<User>> {
    let user = sqlx::query_as::<_, User>(
        "SELECT id, email, password_hash, created_at FROM users WHERE email = $1",
    )
    .bind(email)
    .fetch_optional(pool)
    .await?;

    Ok(user)
}


pub async fn update(
    pool: &PgPool,
    id: Uuid,
    email: Option<&str>,
    password_hash: Option<&str>,
) -> AppResult<Option<User>> {
    sqlx::query_as::<_, User>(
        "UPDATE users
         SET email = COALESCE($2, email),
             password_hash = COALESCE($3, password_hash)
         WHERE id = $1
         RETURNING id, email, password_hash, created_at",
    )
    .bind(id)
    .bind(email)
    .bind(password_hash)
    .fetch_optional(pool)
    .await
    .map_err(map_write_error)
}


pub async fn delete(pool: &PgPool, id: Uuid) -> AppResult<bool> {
    let result = sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected() > 0)
}

fn map_write_error(err: sqlx::Error) -> AppError {
    match unique_violation(&err) {
        Some("users_email_key") => AppError::Conflict("este email já está cadastrado"),
        _ => AppError::Database(err),
    }
}
