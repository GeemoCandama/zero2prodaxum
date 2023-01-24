use axum::{extract::State, Form};
use http::StatusCode;
use serde::Deserialize;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use sqlx::{Transaction, Postgres};

use crate::{
    startup::AppState, 
    domain::{NewSubscriber, SubscriberName, SubscriberEmail},
    email_client::EmailClient,
};

fn generate_subscription_token() -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(25)
        .map(char::from)
        .collect()
}

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
    let mut transaction = match state.db_pool.begin().await {
        Ok(transaction) => transaction,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR,
    };
    let subscriber_id = match insert_subscriber(&mut transaction, &new_subscriber).await {
        Ok(id) => id,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR,
    };
    let subscription_token = generate_subscription_token();
    if store_token(&mut transaction, subscriber_id, &subscription_token).await.is_err() {
        return StatusCode::INTERNAL_SERVER_ERROR;
    }
    if transaction.commit().await.is_err() {
        return StatusCode::INTERNAL_SERVER_ERROR;
    }

    if send_confirmation_email(
        new_subscriber, 
        &state.email_client, 
        &state.base_url,
        &subscription_token,
    )
    .await
    .is_err() {
        return StatusCode::INTERNAL_SERVER_ERROR;
    }
    StatusCode::OK
}

impl TryFrom<FormData> for NewSubscriber {
    type Error = String;

    fn try_from(value: FormData) -> Result<Self, Self::Error> {
        let name = SubscriberName::parse(value.name)?;
        let email = SubscriberEmail::parse(value.email)?;
        Ok(Self { name, email })
    }
}

#[tracing::instrument(
    name = "Send a confirmation email to a new subscriber",
    skip(new_subscriber, email_client, base_url)
)]
pub async fn send_confirmation_email(
    new_subscriber: NewSubscriber,
    email_client: &EmailClient,
    base_url: &str,
    subscription_token: &str,
) -> Result<(), reqwest::Error> {
    let confirmation_link = format!(
        "{}/subscriptions/confirm?subscription_token={}",
        base_url,
        subscription_token,
    );
    let plain_body = format!(
        "Welcome to our newsletter!\nVisit {} to confirm your subscription.",
        confirmation_link
    );
    let html_body = format!(
        "Welcome to our newsletter!<br />\
        Click <a href=\"{}\">here</a> to confirm your subscription.",
        confirmation_link
    );
    email_client
        .send_email(
            &new_subscriber.email,
            "Welcome!",
            &html_body,
            &plain_body,
        )
        .await
}


// insert a new subscriber into the database.
// this procedural macro instuments the function insert_subscriber
// with a span that has the name "insert_subscriber"
#[tracing::instrument(name = "Getting all subscribers", skip(new_subscriber, transaction))]
pub async fn insert_subscriber(
    transaction: &mut Transaction<'_, Postgres>,
    new_subscriber: &NewSubscriber
) -> Result<uuid::Uuid, sqlx::Error> {
    let subscriber_id = uuid::Uuid::new_v4();
    sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at, status)
        VALUES ($1, $2, $3, $4, 'pending_confirmation')
        "#,
        subscriber_id,
        new_subscriber.email.as_ref(),
        new_subscriber.name.as_ref(),
        chrono::Utc::now(),
    )
    .execute(transaction)
    .await
    .map_err(|error| {
        tracing::error!("Failed to execute query: {:?}", error);
        error
    })?;
    Ok(subscriber_id)
}

#[tracing::instrument(
    name = "Store a subscription token in the database",
    skip(subscription_token, transaction)
)]
pub async fn store_token(
    transaction: &mut Transaction<'_, Postgres>,
    subscriber_id: uuid::Uuid,
    subscription_token: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO subscription_tokens (subscription_token, subscriber_id)
        VALUES ($1, $2)
        "#,
        subscription_token,
        subscriber_id
    )
    .execute(transaction)
    .await
    .map_err(|error| {
        tracing::error!("Failed to execute query: {:?}", error);
        error
    })?;
    Ok(())
}
