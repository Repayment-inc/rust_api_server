use axum::{extract::State, Json};
use serde_json::{json, Value};
use sqlx::PgPool;

use crate::{
    error::AppError,
    models,
};

pub async fn register(
    Json(credentials): Json<models::auth::User>,
    // Json(credentials): Json<models::auth::User>,
    State(pool): State<PgPool>,
) -> Result<Json<Value>, AppError> {
    // check if email or password is a blank string
    if credentials.email.is_empty() || credentials.password.is_empty() {
        return Err(AppError::MissingCredential);
    }

    // get the user for the email from database
    let user = sqlx::query_as::<_, models::auth::User>(
        "SELECT email, password FROM users where email = $1",
    )
    .bind(&credentials.email)
    .fetch_optional(&pool)
    .await
    .map_err(|err| {
        dbg!(err);
        AppError::InternalServerError
    })?;

    if let Some(_) = user {
        //if a user with email already exits send error
        return Err(AppError::UserAlreadyExits);
    }

    let result = sqlx::query("INSERT INTO users (email, password) values ($1, $2)")
        .bind(&credentials.email)
        .bind(credentials.password)
        .execute(&pool)
        .await
        .map_err(|_| AppError::InternalServerError)?;
    if result.rows_affected() < 1 {
        Err(AppError::InternalServerError)
    } else {
        Ok(Json(json!({ "msg": "registered successfully" })))
    }
}