use chrono::NaiveDate;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct Certification {
    pub id: Uuid,
    pub profile_id: Uuid,
    pub name: String,
    pub issuing_organization: Option<String>,
    pub issue_date: Option<NaiveDate>,
    pub url: Option<String>,
}
