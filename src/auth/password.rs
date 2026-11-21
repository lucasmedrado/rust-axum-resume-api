//! Hash e verificação de senhas com Argon2.
//!
//! Argon2 é intencionalmente caro em CPU, por isso as versões `async` rodam em
//! `spawn_blocking` para não travar as threads que atendem requisições.

use argon2::Argon2;
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};

use crate::error::{AppError, AppResult};

pub async fn hash_password(password: String) -> AppResult<String> {
    tokio::task::spawn_blocking(move || hash_blocking(&password))
        .await
        .map_err(|e| AppError::Internal(format!("falha na tarefa de hash: {e}")))?
}

pub async fn verify_password(password: String, stored_hash: String) -> AppResult<bool> {
    tokio::task::spawn_blocking(move || verify_blocking(&password, &stored_hash))
        .await
        .map_err(|e| AppError::Internal(format!("falha na tarefa de verificação: {e}")))
}

fn hash_blocking(password: &str) -> AppResult<String> {
    let salt = SaltString::generate(&mut OsRng);

    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|e| AppError::Internal(format!("falha ao gerar hash: {e}")))
}

fn verify_blocking(password: &str, stored_hash: &str) -> bool {
    let Ok(parsed) = PasswordHash::new(stored_hash) else {
        return false;
    };

    Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_valido_confere_com_a_senha_original() {
        let hash = hash_blocking("minha-senha-123").unwrap();

        assert!(verify_blocking("minha-senha-123", &hash));
        assert!(!verify_blocking("senha-errada", &hash));
    }

    #[test]
    fn mesma_senha_gera_hashes_diferentes() {
        let a = hash_blocking("minha-senha-123").unwrap();
        let b = hash_blocking("minha-senha-123").unwrap();

        assert_ne!(a, b, "o salt deve ser aleatório");
    }

    #[test]
    fn hash_armazenado_corrompido_nao_valida() {
        assert!(!verify_blocking("qualquer", "isto-nao-e-um-hash"));
    }
}
