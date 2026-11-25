//! Configuração da aplicação, lida uma única vez das variáveis de ambiente.

use std::env;
use std::net::SocketAddr;

use thiserror::Error;

/// Tamanho mínimo aceito para o segredo do JWT.
const MIN_JWT_SECRET_LEN: usize = 32;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("a variável de ambiente {0} é obrigatória")]
    Missing(&'static str),
    #[error("valor inválido para {name}: {reason}")]
    Invalid { name: &'static str, reason: String },
}

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub jwt_ttl_hours: i64,
    pub bind_addr: SocketAddr,
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        let database_url = required("DATABASE_URL")?; 

        let jwt_secret = required("JWT_SECRET")?;
        if jwt_secret.len() < MIN_JWT_SECRET_LEN {
            return Err(ConfigError::Invalid {
                name: "JWT_SECRET",
                reason: format!("deve ter pelo menos {MIN_JWT_SECRET_LEN} caracteres"),
            });
        }

        let jwt_ttl_hours = parse_or("JWT_TTL_HOURS", 24)?;
        if jwt_ttl_hours <= 0 {
            return Err(ConfigError::Invalid {
                name: "JWT_TTL_HOURS",
                reason: "deve ser maior que zero".to_string(),
            });
        }

        let host: String = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
        let port: u16 = parse_or("PORT", 3000)?;
        let bind_addr =
            format!("{host}:{port}")
                .parse()
                .map_err(|e: std::net::AddrParseError| ConfigError::Invalid {
                    name: "HOST",
                    reason: e.to_string(),
                })?;

        Ok(Self {
            database_url,
            jwt_secret,
            jwt_ttl_hours,
            bind_addr,
        })
    }
}

fn required(name: &'static str) -> Result<String, ConfigError> {
    env::var(name).map_err(|_| ConfigError::Missing(name))
}


fn parse_or<T>(name: &'static str, default: T) -> Result<T, ConfigError>
where
    T: std::str::FromStr,
    T::Err: std::fmt::Display,
{
    match env::var(name) {
        Err(_) => Ok(default),
        Ok(raw) => raw.parse().map_err(|e: T::Err| ConfigError::Invalid {
            name,
            reason: e.to_string(),
        }),
    }
}
