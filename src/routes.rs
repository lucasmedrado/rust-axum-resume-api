use axum::routing::{get, patch, post};
use axum::{Json, Router};
use serde_json::{Value, json};
use tower_http::trace::TraceLayer;

use crate::error::AppError;
use crate::handlers::{
    academic_education_handler, certification_handler, experience_handler, profile_handler,
    skill_handler, social_link_handler, user_handler,
};
use crate::state::AppState;

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health)) 
        // Autenticação e usuário
        .route("/auth/login", post(user_handler::login))
        .route("/users", post(user_handler::register))
        .route(
            "/users/me",
            get(user_handler::get_me)
                .patch(user_handler::update_me)
                .delete(user_handler::delete_me),
        )
        // Perfil (um por usuário)
        .route(
            "/profiles/me",
            post(profile_handler::create)
                .get(profile_handler::get_mine)
                .patch(profile_handler::update)
                .delete(profile_handler::delete),
        )
        // Experiências (várias por perfil)
        .route(
            "/experiences",
            get(experience_handler::list).post(experience_handler::create),
        )
        .route(
            "/experiences/{id}",
            patch(experience_handler::update).delete(experience_handler::delete),
        )
        // Formações acadêmicas (várias por perfil)
        .route(
            "/academic-educations",
            get(academic_education_handler::list).post(academic_education_handler::create),
        )
        .route(
            "/academic-educations/{id}",
            patch(academic_education_handler::update).delete(academic_education_handler::delete),
        )
        // Certificações (várias por perfil)
        .route(
            "/certifications",
            get(certification_handler::list).post(certification_handler::create),
        )
        .route(
            "/certifications/{id}",
            patch(certification_handler::update).delete(certification_handler::delete),
        )
        // Habilidades (várias por perfil)
        .route(
            "/skills",
            get(skill_handler::list).post(skill_handler::create),
        )
        .route(
            "/skills/{id}",
            patch(skill_handler::update).delete(skill_handler::delete),
        )
        // Links sociais (vários por perfil)
        .route(
            "/social-links",
            get(social_link_handler::list).post(social_link_handler::create),
        )
        .route(
            "/social-links/{id}",
            patch(social_link_handler::update).delete(social_link_handler::delete),
        )
        .fallback(|| async { AppError::NotFound("rota") })
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

async fn health() -> Json<Value> {
    Json(json!({ "status": "ok" }))
}
