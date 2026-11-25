use std::sync::LazyLock;

use regex::Regex;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::models::profile::Profile;

static RE_SLUG: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[a-z0-9]+(?:-[a-z0-9]+)*$").expect("regex de slug é válida"));

#[derive(Debug, Deserialize, Validate)]
pub struct CreateProfileRequest {
    #[validate(length(
        min = 3,
        max = 100,
        message = "Nome completo deve ter entre 3 e 100 caracteres"
    ))]
    pub full_name: String,

    #[validate(length(
        min = 3,
        max = 100,
        message = "Localização deve ter entre 3 e 100 caracteres"
    ))]
    pub localization: String,

    #[validate(length(
        min = 15,
        max = 500,
        message = "O objetivo profissional deve ter entre 15 e 500 caracteres"
    ))]
    pub professional_objective: Option<String>,

    #[validate(
        length(min = 3, max = 50, message = "O slug deve ter entre 3 e 50 caracteres"),
        regex(
            path = *RE_SLUG,
            message = "O slug deve conter apenas letras minúsculas, números e hifens (ex: meu-perfil-123)"
        )
    )]
    pub slug: String,

    #[validate(length(
        min = 15,
        max = 500,
        message = "A bio deve ter entre 15 e 500 caracteres"
    ))]
    pub bio: Option<String>,
}


#[derive(Debug, Deserialize, Validate)]
pub struct UpdateProfileRequest {
    #[validate(length(
        min = 3,
        max = 100,
        message = "Nome completo deve ter entre 3 e 100 caracteres"
    ))]
    pub full_name: Option<String>,

    #[validate(length(
        min = 3,
        max = 100,
        message = "Localização deve ter entre 3 e 100 caracteres"
    ))]
    pub localization: Option<String>,

    #[validate(length(
        min = 15,
        max = 500,
        message = "O objetivo profissional deve ter entre 15 e 500 caracteres"
    ))]
    pub professional_objective: Option<String>,

    #[validate(
        length(min = 3, max = 50, message = "O slug deve ter entre 3 e 50 caracteres"),
        regex(
            path = *RE_SLUG,
            message = "O slug deve conter apenas letras minúsculas, números e hifens (ex: meu-perfil-123)"
        )
    )]
    pub slug: Option<String>,

    #[validate(length(
        min = 15,
        max = 500,
        message = "A bio deve ter entre 15 e 500 caracteres"
    ))]
    pub bio: Option<String>,

    pub is_public: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct ProfileResponse {
    pub id: Uuid,
    pub full_name: String,
    pub localization: String,
    pub professional_objective: Option<String>,
    pub slug: String,
    pub bio: Option<String>,
    pub is_public: bool,
}

impl From<Profile> for ProfileResponse {
    fn from(profile: Profile) -> Self {
        Self {
            id: profile.id,
            full_name: profile.full_name,
            localization: profile.localization,
            professional_objective: profile.professional_objective,
            slug: profile.slug,
            bio: profile.bio,
            is_public: profile.is_public,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request_with_slug(slug: &str) -> CreateProfileRequest {
        CreateProfileRequest {
            full_name: "Maria Silva".into(),
            localization: "São Paulo, SP".into(),
            professional_objective: None,
            slug: slug.into(),
            bio: None,
        }
    }

    #[test]
    fn slugs_validos() {
        for slug in ["maria", "maria-silva", "maria-silva-2024", "abc123"] {
            assert!(request_with_slug(slug).validate().is_ok(), "slug: {slug}");
        }
    }

    #[test]
    fn slugs_invalidos() {
        for slug in [
            "Maria",
            "maria silva",
            "-maria",
            "maria-",
            "maria--silva",
            "ma",
            "maria_silva",
        ] {
            assert!(request_with_slug(slug).validate().is_err(), "slug: {slug}");
        }
    }

    #[test]
    fn update_vazio_e_valido() {
        let req = UpdateProfileRequest {
            full_name: None,
            localization: None,
            professional_objective: None,
            slug: None,
            bio: None,
            is_public: None,
        };

        assert!(req.validate().is_ok());
    }
}
