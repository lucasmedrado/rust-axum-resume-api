use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct Skill {
    pub id: Uuid,
    pub profile_id: Uuid,
    pub name: String,
    pub skill_type: String,
    pub level: Option<String>,
}
