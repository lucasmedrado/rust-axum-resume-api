use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::models::user::User;

const PASSWORD_MIN: u64 = 8;

const PASSWORD_MAX: u64 = 128;

#[derive(Debug, Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(email(message = "Email inválido"))]
    pub email: String,

    #[validate(length(
        min = PASSWORD_MIN,
        max = PASSWORD_MAX,
        message = "A senha deve ter entre 8 e 128 caracteres"
    ))]
    pub password: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(email(message = "Email inválido"))]
    pub email: String,

    #[validate(length(min = 1, max = PASSWORD_MAX, message = "Informe a senha"))]
    pub password: String,
}


#[derive(Debug, Deserialize, Validate)]
pub struct UpdateUserRequest {
    #[validate(length(min = 1, max = PASSWORD_MAX, message = "Informe a senha atual"))]
    pub current_password: String,

    #[validate(email(message = "Email inválido"))]
    pub email: Option<String>,

    #[validate(length(
        min = PASSWORD_MIN,
        max = PASSWORD_MAX,
        message = "A nova senha deve ter entre 8 e 128 caracteres"
    ))]
    pub new_password: Option<String>,
}


#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub email: String,
    pub created_at: DateTime<Utc>,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            email: user.email,
            created_at: user.created_at,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub user: UserResponse,
    pub token: String,
}

#[derive(Debug, Serialize)]
pub struct TokenResponse {
    pub token: String,
}
