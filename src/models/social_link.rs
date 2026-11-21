use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct SocialLink {
    pub id: Uuid,
    pub profile_id: Uuid,
    pub platform_name: String,
    pub url: String,
}
