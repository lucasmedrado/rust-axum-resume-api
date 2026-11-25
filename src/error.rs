

use std::collections::HashMap;

use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;
use thiserror::Error;
use validator::ValidationErrors;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("{0} não encontrado(a)")]
    NotFound(&'static str),

    #[error("{0}")]
    Conflict(&'static str),

    
    #[error("{0}")]
    BadRequest(String),

    
    #[error("{0}")]
    Unprocessable(&'static str),

    #[error("dados inválidos")]
    Validation(#[from] ValidationErrors),

    #[error("email ou senha incorretos")]
    InvalidCredentials,

    #[error("token ausente ou inválido")]
    InvalidToken,

    #[error("token expirado")]
    ExpiredToken,

    #[error("erro de banco de dados: {0}")]
    Database(#[from] sqlx::Error),

    #[error("erro interno: {0}")]
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match &self {
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::Conflict(_) => StatusCode::CONFLICT,
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::Unprocessable(_) | AppError::Validation(_) => {
                StatusCode::UNPROCESSABLE_ENTITY
            }
            AppError::InvalidCredentials | AppError::InvalidToken | AppError::ExpiredToken => {
                StatusCode::UNAUTHORIZED
            }
            AppError::Database(_) | AppError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        let body = match &self {
            
            AppError::Database(_) | AppError::Internal(_) => {
                tracing::error!(error = %self, "erro interno");
                json!({ "error": "erro interno do servidor" })
            }
            AppError::Validation(errors) => json!({
                "error": self.to_string(),
                "fields": field_messages(errors),
            }),
            _ => json!({ "error": self.to_string() }),
        };

        (status, Json(body)).into_response()
    }
}


fn field_messages(errors: &ValidationErrors) -> HashMap<String, Vec<String>> {
    errors
        .field_errors()
        .into_iter()
        .map(|(field, errs)| {
            let messages = errs
                .iter()
                .map(|e| match &e.message {
                    Some(message) => message.to_string(),
                    None => format!("valor inválido ({})", e.code),
                })
                .collect();
            (field.to_string(), messages)
        })
        .collect()
}


pub fn unique_violation(err: &sqlx::Error) -> Option<&str> {
    let db_err = err.as_database_error()?;
    if db_err.is_unique_violation() {
        db_err.constraint()
    } else {
        None
    }
}

pub fn is_foreign_key_violation(err: &sqlx::Error) -> bool {
    err.as_database_error()
        .is_some_and(|e| e.is_foreign_key_violation())
}

pub fn is_check_violation(err: &sqlx::Error) -> bool {
    err.as_database_error()
        .is_some_and(|e| e.is_check_violation())
}
