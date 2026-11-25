use chrono::{NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::{Validate, ValidationError};

use crate::models::academic_education::AcademicEducation;

#[derive(Debug, Deserialize, Validate)]
#[validate(schema(function = "validate_period"))]
pub struct CreateAcademicEducationRequest {
    #[validate(length(
        min = 2,
        max = 150,
        message = "O nome da instituição deve ter entre 2 e 150 caracteres"
    ))]
    pub institution: String,

    #[validate(length(
        min = 2,
        max = 100,
        message = "O grau deve ter entre 2 e 100 caracteres"
    ))]
    pub degree: Option<String>,

    #[validate(length(
        min = 2,
        max = 150,
        message = "A área de estudo deve ter entre 2 e 150 caracteres"
    ))]
    pub field_of_study: Option<String>,

    #[validate(custom(function = "validate_not_in_future"))]
    pub start_date: Option<NaiveDate>,

    pub end_date: Option<NaiveDate>,
}


#[derive(Debug, Deserialize, Validate)]
#[validate(schema(function = "validate_update_period"))]
pub struct UpdateAcademicEducationRequest {
    #[validate(length(
        min = 2,
        max = 150,
        message = "O nome da instituição deve ter entre 2 e 150 caracteres"
    ))]
    pub institution: Option<String>,

    #[validate(length(
        min = 2,
        max = 100,
        message = "O grau deve ter entre 2 e 100 caracteres"
    ))]
    pub degree: Option<String>,

    #[validate(length(
        min = 2,
        max = 150,
        message = "A área de estudo deve ter entre 2 e 150 caracteres"
    ))]
    pub field_of_study: Option<String>,

    #[validate(custom(function = "validate_not_in_future"))]
    pub start_date: Option<NaiveDate>,

    pub end_date: Option<NaiveDate>,
}

#[derive(Debug, Serialize)]
pub struct AcademicEducationResponse {
    pub id: Uuid,
    pub institution: String,
    pub degree: Option<String>,
    pub field_of_study: Option<String>,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
}

impl From<AcademicEducation> for AcademicEducationResponse {
    fn from(education: AcademicEducation) -> Self {
        Self {
            id: education.id,
            institution: education.institution,
            degree: education.degree,
            field_of_study: education.field_of_study,
            start_date: education.start_date,
            end_date: education.end_date,
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

fn validate_period(request: &CreateAcademicEducationRequest) -> Result<(), ValidationError> {
    match (request.start_date, request.end_date) {
        (Some(start), Some(end)) if end < start => Err(period_error()),
        _ => Ok(()),
    }
}

fn validate_update_period(request: &UpdateAcademicEducationRequest) -> Result<(), ValidationError> {
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

    fn create(start: Option<NaiveDate>, end: Option<NaiveDate>) -> CreateAcademicEducationRequest {
        CreateAcademicEducationRequest {
            institution: "Fatec".into(),
            degree: Some("Tecnólogo".into()),
            field_of_study: Some("ADS".into()),
            start_date: start,
            end_date: end,
        }
    }

    #[test]
    fn formacao_em_andamento_ou_sem_datas_e_valida() {
        assert!(create(Some(date(2020, 1, 1)), None).validate().is_ok());
        assert!(create(None, None).validate().is_ok());
    }

    #[test]
    fn periodo_com_fim_depois_do_inicio_e_valido() {
        assert!(
            create(Some(date(2020, 1, 1)), Some(date(2023, 12, 15)))
                .validate()
                .is_ok()
        );
    }

    #[test]
    fn fim_antes_do_inicio_e_invalido() {
        assert!(
            create(Some(date(2022, 1, 1)), Some(date(2020, 1, 1)))
                .validate()
                .is_err()
        );
    }

    #[test]
    fn inicio_no_futuro_e_invalido() {
        let amanha = Utc::now().date_naive() + Duration::days(1);
        assert!(create(Some(amanha), None).validate().is_err());
    }
}
