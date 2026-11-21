use chrono::{Duration, Utc};
use jsonwebtoken::errors::ErrorKind;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::{AppError, AppResult};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    /// Id do usuário dono do token.
    pub sub: Uuid,
    /// Momento de emissão (segundos desde a época Unix).
    pub iat: i64,
    /// Momento de expiração (segundos desde a época Unix).
    pub exp: i64,
}

pub fn create_token(user_id: Uuid, secret: &str, ttl_hours: i64) -> AppResult<String> {
    let now = Utc::now();
    let claims = Claims {
        sub: user_id,
        iat: now.timestamp(),
        exp: (now + Duration::hours(ttl_hours)).timestamp(),
    };

    encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| AppError::Internal(format!("falha ao gerar token: {e}")))
}

pub fn verify_token(token: &str, secret: &str) -> AppResult<Claims> {
    // Aceita somente HS256 e valida `exp` (padrão da crate).
    let validation = Validation::new(Algorithm::HS256);

    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )
    .map(|data| data.claims)
    .map_err(|err| match err.kind() {
        ErrorKind::ExpiredSignature => AppError::ExpiredToken,
        _ => AppError::InvalidToken,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const SECRET: &str = "segredo-de-teste-com-mais-de-32-caracteres";

    #[test]
    fn token_gerado_e_validado_com_sucesso() {
        let id = Uuid::from_u128(42);
        let token = create_token(id, SECRET, 1).unwrap();

        let claims = verify_token(&token, SECRET).unwrap();

        assert_eq!(claims.sub, id);
        assert!(claims.exp > claims.iat);
    }

    #[test]
    fn segredo_diferente_e_rejeitado() {
        let token = create_token(Uuid::from_u128(42), SECRET, 1).unwrap();

        let result = verify_token(&token, "outro-segredo-completamente-diferente");

        assert!(matches!(result, Err(AppError::InvalidToken)));
    }

    #[test]
    fn token_expirado_e_rejeitado() {
        // TTL bem negativo gera um token já vencido (a crate tolera 60s de diferença).
        let token = create_token(Uuid::from_u128(42), SECRET, -1).unwrap();

        let result = verify_token(&token, SECRET);

        assert!(matches!(result, Err(AppError::ExpiredToken)));
    }

    #[test]
    fn lixo_nao_e_um_token_valido() {
        assert!(matches!(
            verify_token("isto.nao.e-um-jwt", SECRET),
            Err(AppError::InvalidToken)
        ));
    }
}
