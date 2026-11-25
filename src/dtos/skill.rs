use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::{Validate, ValidationError};

use crate::models::skill::Skill;

#[derive(Debug, Deserialize, Validate)]
pub struct CreateSkillRequest {
    #[validate(length(
        min = 1,
        max = 100,
        message = "O nome da habilidade deve ter entre 1 e 100 caracteres"
    ))]
    pub name: String,

    #[validate(custom(function = "validate_skill_type"))]
    pub skill_type: String,

    #[validate(custom(function = "validate_skill_level"))]
    pub level: Option<String>,
}


#[derive(Debug, Deserialize, Validate)]
pub struct UpdateSkillRequest {
    #[validate(length(
        min = 1,
        max = 100,
        message = "O nome da habilidade deve ter entre 1 e 100 caracteres"
    ))]
    pub name: Option<String>,

    #[validate(custom(function = "validate_skill_type"))]
    pub skill_type: Option<String>,

    #[validate(custom(function = "validate_skill_level"))]
    pub level: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SkillResponse {
    pub id: Uuid,
    pub name: String,
    pub skill_type: String,
    pub level: Option<String>,
}

impl From<Skill> for SkillResponse {
    fn from(skill: Skill) -> Self {
        Self {
            id: skill.id,
            name: skill.name,
            skill_type: skill.skill_type,
            level: skill.level,
        }
    }
}

fn validate_skill_type(skill_type: &str) -> Result<(), ValidationError> {
    match skill_type {
        "hard" | "soft" => Ok(()),
        _ => {
            let mut error = ValidationError::new("invalid_skill_type");
            error.message = Some("O tipo da habilidade deve ser 'hard' ou 'soft'".into());
            Err(error)
        }
    }
}

fn validate_skill_level(level: &str) -> Result<(), ValidationError> {
    match level {
        "beginner" | "intermediate" | "advanced" | "expert" => Ok(()),
        _ => {
            let mut error = ValidationError::new("invalid_skill_level");
            error.message = Some(
                "O nível deve ser 'beginner', 'intermediate', 'advanced' ou 'expert'".into(),
            );
            Err(error)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tipos_e_niveis_validos() {
        for skill_type in ["hard", "soft"] {
            for level in [
                None,
                Some("beginner".into()),
                Some("intermediate".into()),
                Some("advanced".into()),
                Some("expert".into()),
            ] {
                let req = CreateSkillRequest {
                    name: "Rust".into(),
                    skill_type: skill_type.into(),
                    level,
                };
                assert!(req.validate().is_ok());
            }
        }
    }

    #[test]
    fn tipo_ou_nivel_invalido_falha() {
        let bad_type = CreateSkillRequest {
            name: "Rust".into(),
            skill_type: "technical".into(),
            level: None,
        };
        assert!(bad_type.validate().is_err());

        let bad_level = CreateSkillRequest {
            name: "Rust".into(),
            skill_type: "hard".into(),
            level: Some("master".into()),
        };
        assert!(bad_level.validate().is_err());
    }
}
