use sqlx::PgPool;
use uuid::Uuid;

use crate::auth::{jwt, password};
use crate::dtos::user::{LoginRequest, RegisterRequest, UpdateUserRequest};
use crate::error::{AppError, AppResult};
use crate::models::user::User;
use crate::repositories::user_repository;
use crate::state::AppState;

/// Emails são tratados sem diferenciar maiúsculas/minúsculas.
fn normalize_email(email: &str) -> String {
    email.trim().to_lowercase()
}

/// Cadastra o usuário e já devolve um token, para ele não precisar fazer login em seguida.
pub async fn register(state: &AppState, request: RegisterRequest) -> AppResult<(User, String)> {
    let email = normalize_email(&request.email);
    let password_hash = password::hash_password(request.password).await?;

    let user = user_repository::create(&state.pool, &email, &password_hash).await?;
    let token = issue_token(state, user.id)?;

    Ok((user, token))
}

pub async fn login(state: &AppState, request: LoginRequest) -> AppResult<String> {
    let email = normalize_email(&request.email);

    // Mesma resposta para "email inexistente" e "senha errada",
    // para não revelar quais emails estão cadastrados.
    let user = user_repository::find_by_email(&state.pool, &email)
        .await?
        .ok_or(AppError::InvalidCredentials)?;

    if !password::verify_password(request.password, user.password_hash).await? {
        return Err(AppError::InvalidCredentials);
    }

    issue_token(state, user.id)
}

pub async fn get(pool: &PgPool, user_id: Uuid) -> AppResult<User> {
    user_repository::find_by_id(pool, user_id)
        .await?
        .ok_or(AppError::NotFound("usuário"))
}

pub async fn update(pool: &PgPool, user_id: Uuid, request: UpdateUserRequest) -> AppResult<User> {
    let user = get(pool, user_id).await?;

    if !password::verify_password(request.current_password, user.password_hash).await? {
        return Err(AppError::InvalidCredentials);
    }

    let new_email = request.email.as_deref().map(normalize_email);
    let new_password_hash = match request.new_password {
        Some(new_password) => Some(password::hash_password(new_password).await?),
        None => None,
    };

    user_repository::update(
        pool,
        user_id,
        new_email.as_deref(),
        new_password_hash.as_deref(),
    )
    .await?
    .ok_or(AppError::NotFound("usuário"))
}

pub async fn delete(pool: &PgPool, user_id: Uuid) -> AppResult<()> {
    if user_repository::delete(pool, user_id).await? {
        Ok(())
    } else {
        Err(AppError::NotFound("usuário"))
    }
}

fn issue_token(state: &AppState, user_id: Uuid) -> AppResult<String> {
    jwt::create_token(
        user_id,
        &state.config.jwt_secret,
        state.config.jwt_ttl_hours,
    )
}
