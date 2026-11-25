use chrono::{NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::{Validate, ValidationError};

use crate::models::experience::Experience;

#[derive(Debug, Deserialize, Validate)]
#[validate(schema(function = "validate_period"))]
pub struct CreateExperienceRequest {
    #[validate(length(
        min = 2,
        max = 150,
        message = "O nome da empresa deve ter entre 2 e 150 caracteres"
    ))]
    pub company_name: String,

    #[validate(length(
        min = 2,
        max = 150,
        message = "O cargo deve ter entre 2 e 150 caracteres"
    ))]
    pub position: String,

    #[validate(custom(function = "validate_not_in_future"))]
    pub start_date: NaiveDate,

    pub end_date: Option<NaiveDate>,

    #[validate(length(max = 2000, message = "A descrição deve ter no máximo 2000 caracteres"))]
    pub description: Option<String>,
}


#[derive(Debug, Deserialize, Validate)]
#[validate(schema(function = "validate_update_period"))]
pub struct UpdateExperienceRequest {
    #[validate(length(
        min = 2,
        max = 150,
        message = "O nome da empresa deve ter entre 2 e 150 caracteres"
    ))]
    pub company_name: Option<String>,

    #[validate(length(
        min = 2,
        max = 150,
        message = "O cargo deve ter entre 2 e 150 caracteres"
    ))]
    pub position: Option<String>,

    #[validate(custom(function = "validate_not_in_future"))]
    pub start_date: Option<NaiveDate>,

    pub end_date: Option<NaiveDate>,

    #[validate(length(max = 2000, message = "A descrição deve ter no máximo 2000 caracteres"))]
    pub description: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ExperienceResponse {
    pub id: Uuid,
    pub company_name: String,
    pub position: String,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub description: Option<String>,
}

impl From<Experience> for ExperienceResponse {
    fn from(experience: Experience) -> Self {
        Self {
            id: experience.id,
            company_name: experience.company_name,
            position: experience.position,
            start_date: experience.start_date,
            end_date: experience.end_date,
            description: experience.description,
        }
    }
}

fn validate_not_in_future(date: &NaiveDate) -> Result<(), ValidationError> {
    if *date > Utc::now().date_naive() {
        let mut error = ValidationError::new("future_date");
        error.message = Some("A data de início não pode estar no futuro".into());
        return Err(error);
    }
    Ok(())
}

fn period_error() -> ValidationError {
    let mut error = ValidationError::new("invalid_period");
    error.message = Some("A data final não pode ser anterior à data inicial".into());
    error
}

fn validate_period(request: &CreateExperienceRequest) -> Result<(), ValidationError> {
    match request.end_date {
        Some(end) if end < request.start_date => Err(period_error()),
        _ => Ok(()),
    }
}


fn validate_update_period(request: &UpdateExperienceRequest) -> Result<(), ValidationError> {
    match (request.start_date, request.end_date) {
        (Some(start), Some(end)) if end < start => Err(period_error()),
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use chrono::Duration;

    use super::*;

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    fn create(start: NaiveDate, end: Option<NaiveDate>) -> CreateExperienceRequest {
        CreateExperienceRequest {
            company_name: "Acme".into(),
            position: "Desenvolvedora".into(),
            start_date: start,
            end_date: end,
            description: None,
        }
    }

    #[test]
    fn experiencia_em_andamento_e_valida() {
        assert!(create(date(2020, 1, 1), None).validate().is_ok());
    }

    #[test]
    fn periodo_com_fim_depois_do_inicio_e_valido() {
        assert!(
            create(date(2020, 1, 1), Some(date(2022, 6, 30)))
                .validate()
                .is_ok()
        );
    }

    #[test]
    fn fim_antes_do_inicio_e_invalido() {
        let result = create(date(2022, 1, 1), Some(date(2020, 1, 1))).validate();

        assert!(result.is_err());
    }

    #[test]
    fn inicio_no_futuro_e_invalido() {
        let amanha = Utc::now().date_naive() + Duration::days(1);

        assert!(create(amanha, None).validate().is_err());
    }

    #[test]
    fn update_so_com_uma_data_nao_compara_periodo() {
        let req = UpdateExperienceRequest {
            company_name: None,
            position: None,
            start_date: None,
            end_date: Some(date(2000, 1, 1)),
            description: None,
        };

        assert!(req.validate().is_ok());
    }
}
