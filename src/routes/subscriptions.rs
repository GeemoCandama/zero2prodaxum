use axum::{
    Form,
    extract::State,
};
use serde::Deserialize;
use http::StatusCode;

use crate::startup::AppState;

#[derive(Deserialize)]
pub struct FormData {
    name: String,
    email: String,
}

pub async fn subscribe(
    State(state): State<AppState>,
    Form(form_data): Form<FormData>,
) -> StatusCode {
    match sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)
        "#,
        uuid::Uuid::new_v4(),
        form_data.email,
        form_data.name,
        chrono::Utc::now(),
    )
    .execute(&state.db_pool)
    .await {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}
