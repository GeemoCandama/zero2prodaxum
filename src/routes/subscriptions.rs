use axum::{extract::State, Form};
use http::StatusCode;
use serde::Deserialize;

use crate::startup::AppState;

#[derive(Deserialize)]
pub struct FormData {
    name: String,
    email: String,
}

// #[tracing::instrument] creates a span at the beginning of the function invocation and automat-
// ically attaches all arguments passed to the function to the context of the span
// This function is the handler for the POST /subscriptions route
// It takes the form data and the state and returns a response
// It creates a span for the request and adds the form data as attributes
#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(form_data, state),
    fields(
        name = %form_data.name,
        email = %form_data.email,
    )
)]
pub async fn subscribe(
    State(state): State<AppState>,
    Form(form_data): Form<FormData>,
) -> StatusCode {
    match insert_subscriber(&state.db_pool, &form_data).await {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

// insert a new subscriber into the database.
// this procedural macro instuments the function insert_subscriber
// with a span that has the name "insert_subscriber"
#[tracing::instrument(name = "Getting all subscribers", skip(form, pool))]
pub async fn insert_subscriber(pool: &sqlx::PgPool, form: &FormData) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)
        "#,
        uuid::Uuid::new_v4(),
        form.email,
        form.name,
        chrono::Utc::now(),
    )
    .execute(pool)
    .await
    .map_err(|error| {
        tracing::error!("Failed to execute query: {:?}", error);
        error
    })?;
    Ok(())
}
