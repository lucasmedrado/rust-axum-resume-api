use chrono::NaiveDate;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct AcademicEducation {
    pub id: Uuid,
    pub profile_id: Uuid,
    pub institution: String,
    pub degree: Option<String>,
    pub field_of_study: Option<String>,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
}
