use axum::{extract::State, Form};
use http::StatusCode;
use serde::Deserialize;

use crate::{startup::AppState, domain::{NewSubscriber, SubscriberName, SubscriberEmail}};

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
    let new_subscriber = match form_data.try_into() {
        Ok(subscriber) => subscriber,
        Err(_) => return StatusCode::BAD_REQUEST,
    };
    match insert_subscriber(&state.db_pool, &new_subscriber).await {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

impl TryFrom<FormData> for NewSubscriber {
    type Error = String;

    fn try_from(value: FormData) -> Result<Self, Self::Error> {
        let name = SubscriberName::parse(value.name)?;
        let email = SubscriberEmail::parse(value.email)?;
        Ok(Self { name, email })
    }
}

// insert a new subscriber into the database.
// this procedural macro instuments the function insert_subscriber
// with a span that has the name "insert_subscriber"
#[tracing::instrument(name = "Getting all subscribers", skip(new_subscriber, pool))]
pub async fn insert_subscriber(
    pool: &sqlx::PgPool, 
    new_subscriber: &NewSubscriber
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)
        "#,
        uuid::Uuid::new_v4(),
        new_subscriber.email.as_ref(),
        new_subscriber.name.as_ref(),
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
