use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::{Validate, ValidationError};

use crate::models::social_link::SocialLink;

#[derive(Debug, Deserialize, Validate)]
pub struct CreateSocialLinkRequest {
    #[validate(length(
        min = 2,
        max = 50,
        message = "O nome da plataforma deve ter entre 2 e 50 caracteres"
    ))]
    pub platform_name: String,

    #[validate(
        length(max = 500, message = "A URL deve ter no máximo 500 caracteres"),
        url(message = "A URL do link social é inválida"),
        custom(function = "validate_http_scheme")
    )]
    pub url: String,
}

/// Todos os campos são opcionais: só o que for enviado é alterado.
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateSocialLinkRequest {
    #[validate(length(
        min = 2,
        max = 50,
        message = "O nome da plataforma deve ter entre 2 e 50 caracteres"
    ))]
    pub platform_name: Option<String>,

    #[validate(
        length(max = 500, message = "A URL deve ter no máximo 500 caracteres"),
        url(message = "A URL do link social é inválida"),
        custom(function = "validate_http_scheme")
    )]
    pub url: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SocialLinkResponse {
    pub id: Uuid,
    pub platform_name: String,
    pub url: String,
}

impl From<SocialLink> for SocialLinkResponse {
    fn from(link: SocialLink) -> Self {
        Self {
            id: link.id,
            platform_name: link.platform_name,
            url: link.url,
        }
    }
}

fn validate_http_scheme(url: &str) -> Result<(), ValidationError> {
    let lower = url.to_ascii_lowercase();
    if !lower.starts_with("http://") && !lower.starts_with("https://") {
        let mut error = ValidationError::new("invalid_url_scheme");
        error.message = Some("A URL deve começar com http:// ou https://".into());
        return Err(error);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn link_social_valido() {
        let req = CreateSocialLinkRequest {
            platform_name: "GitHub".into(),
            url: "https://github.com/lucasmedrado".into(),
        };
        assert!(req.validate().is_ok());
    }

    #[test]
    fn link_social_sem_http_e_rejeitado() {
        let req = CreateSocialLinkRequest {
            platform_name: "GitHub".into(),
            url: "mailto:teste@exemplo.com".into(),
        };
        assert!(req.validate().is_err());
    }
}
