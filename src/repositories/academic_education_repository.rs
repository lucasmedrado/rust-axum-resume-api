use sqlx::PgPool;
use uuid::Uuid;

use crate::dtos::academic_education::{
    CreateAcademicEducationRequest, UpdateAcademicEducationRequest,
};
use crate::error::{AppError, AppResult, is_check_violation};
use crate::models::academic_education::AcademicEducation;



pub async fn list_by_user_id(pool: &PgPool, user_id: Uuid) -> AppResult<Vec<AcademicEducation>> {
    let educations = sqlx::query_as::<_, AcademicEducation>(
        "SELECT ae.id, ae.profile_id, ae.institution, ae.degree, ae.field_of_study, ae.start_date, ae.end_date
         FROM academic_educations ae
         JOIN profiles p ON p.id = ae.profile_id
         WHERE p.user_id = $1
         ORDER BY ae.start_date DESC NULLS LAST, ae.id",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    Ok(educations)
}


pub async fn create(
    pool: &PgPool,
    user_id: Uuid,
    request: &CreateAcademicEducationRequest,
) -> AppResult<Option<AcademicEducation>> {
    sqlx::query_as::<_, AcademicEducation>(
        "INSERT INTO academic_educations (profile_id, institution, degree, field_of_study, start_date, end_date)
         SELECT p.id, $2, $3, $4, $5, $6
         FROM profiles p
         WHERE p.user_id = $1
         RETURNING id, profile_id, institution, degree, field_of_study, start_date, end_date",
    )
    .bind(user_id)
    .bind(&request.institution)
    .bind(request.degree.as_deref())
    .bind(request.field_of_study.as_deref())
    .bind(request.start_date)
    .bind(request.end_date)
    .fetch_optional(pool)
    .await
    .map_err(map_write_error)
}


pub async fn update(
    pool: &PgPool,
    user_id: Uuid,
    education_id: Uuid,
    request: &UpdateAcademicEducationRequest,
) -> AppResult<Option<AcademicEducation>> {
    sqlx::query_as::<_, AcademicEducation>(
        "UPDATE academic_educations ae
         SET institution = COALESCE($3, ae.institution),
             degree = COALESCE($4, ae.degree),
             field_of_study = COALESCE($5, ae.field_of_study),
             start_date = COALESCE($6, ae.start_date),
             end_date = COALESCE($7, ae.end_date)
         FROM profiles p
         WHERE ae.id = $2 AND ae.profile_id = p.id AND p.user_id = $1
         RETURNING ae.id, ae.profile_id, ae.institution, ae.degree, ae.field_of_study, ae.start_date, ae.end_date",
    )
    .bind(user_id)
    .bind(education_id)
    .bind(request.institution.as_deref())
    .bind(request.degree.as_deref())
    .bind(request.field_of_study.as_deref())
    .bind(request.start_date)
    .bind(request.end_date)
    .fetch_optional(pool)
    .await
    .map_err(map_write_error)
}


pub async fn delete(pool: &PgPool, user_id: Uuid, education_id: Uuid) -> AppResult<bool> {
    let result = sqlx::query(
        "DELETE FROM academic_educations ae
         USING profiles p
         WHERE ae.id = $2 AND ae.profile_id = p.id AND p.user_id = $1",
    )
    .bind(user_id)
    .bind(education_id)
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

fn map_write_error(err: sqlx::Error) -> AppError {
    if is_check_violation(&err) {
        AppError::Unprocessable("A data final não pode ser anterior à data inicial")
    } else {
        AppError::Database(err)
    }
}
