use chrono::{NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::{Validate, ValidationError};

use crate::models::certification::Certification;

#[derive(Debug, Deserialize, Validate)]
pub struct CreateCertificationRequest {
    #[validate(length(
        min = 2,
        max = 150,
        message = "O nome da certificação deve ter entre 2 e 150 caracteres"
    ))]
    pub name: String,

    #[validate(length(
        min = 2,
        max = 150,
        message = "A organização emissora deve ter entre 2 e 150 caracteres"
    ))]
    pub issuing_organization: Option<String>,

    #[validate(custom(function = "validate_not_in_future"))]
    pub issue_date: Option<NaiveDate>,

    #[validate(
        length(max = 500, message = "A URL deve ter no máximo 500 caracteres"),
        url(message = "A URL do certificado é inválida"),
        custom(function = "validate_http_scheme")
    )]
    pub url: Option<String>,
}


#[derive(Debug, Deserialize, Validate)]
pub struct UpdateCertificationRequest {
    #[validate(length(
        min = 2,
        max = 150,
        message = "O nome da certificação deve ter entre 2 e 150 caracteres"
    ))]
    pub name: Option<String>,

    #[validate(length(
        min = 2,
        max = 150,
        message = "A organização emissora deve ter entre 2 e 150 caracteres"
    ))]
    pub issuing_organization: Option<String>,

    #[validate(custom(function = "validate_not_in_future"))]
    pub issue_date: Option<NaiveDate>,

    #[validate(
        length(max = 500, message = "A URL deve ter no máximo 500 caracteres"),
        url(message = "A URL do certificado é inválida"),
        custom(function = "validate_http_scheme")
    )]
    pub url: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CertificationResponse {
    pub id: Uuid,
    pub name: String,
    pub issuing_organization: Option<String>,
    pub issue_date: Option<NaiveDate>,
    pub url: Option<String>,
}

impl From<Certification> for CertificationResponse {
    fn from(cert: Certification) -> Self {
        Self {
            id: cert.id,
            name: cert.name,
            issuing_organization: cert.issuing_organization,
            issue_date: cert.issue_date,
            url: cert.url,
        }
    }
}

fn validate_not_in_future(date: &NaiveDate) -> Result<(), ValidationError> {
    if *date > Utc::now().date_naive() {
        let mut error = ValidationError::new("future_date");
        error.message = Some("A data de emissão não pode estar no futuro".into());
        return Err(error);
    }
    Ok(())
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
    fn certificacao_com_url_http_ou_https_e_valida() {
        for url in [
            Some("https://example.com/cert/123".into()),
            Some("http://example.com/cert".into()),
            None,
        ] {
            let req = CreateCertificationRequest {
                name: "AWS Certified".into(),
                issuing_organization: Some("Amazon".into()),
                issue_date: None,
                url,
            };
            assert!(req.validate().is_ok());
        }
    }

    #[test]
    fn url_com_esquema_invalido_e_rejeitada() {
        let req = CreateCertificationRequest {
            name: "AWS Certified".into(),
            issuing_organization: None,
            issue_date: None,
            url: Some("ftp://example.com/cert".into()),
        };
        assert!(req.validate().is_err());
    }
}
