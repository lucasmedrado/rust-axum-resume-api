use chrono::NaiveDate;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct Experience {
    pub id: Uuid,
    pub profile_id: Uuid,
    pub company_name: String,
    pub position: String,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub description: Option<String>,
}
