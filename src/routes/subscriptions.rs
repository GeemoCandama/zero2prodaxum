use axum::{extract::State, Form};
use axum::response::{IntoResponse, Response};
use http::StatusCode;
use serde::Deserialize;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use sqlx::{Transaction, Postgres};
use anyhow::Context;

use crate::{
    startup::AppState, 
    domain::{NewSubscriber, SubscriberName, SubscriberEmail},
    email_client::EmailClient,
};

#[derive(thiserror::Error)]
pub enum SubscribeError {
    #[error("{0}")]
    ValidationError(String),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl std::fmt::Debug for SubscribeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

pub fn error_chain_fmt(
    e: &impl std::error::Error,
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    write!(f, "{}\n", e)?;
    let mut source = e.source();
    while let Some(e) = source {
        write!(f, "Caused by: {}\n", e)?;
        source = e.source();
    }
    Ok(())
}

impl SubscribeError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::ValidationError(_) => StatusCode::BAD_REQUEST,
            Self::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl From<String> for SubscribeError {
    fn from(err: String) -> Self {
        Self::ValidationError(err)
    }
}

impl IntoResponse for SubscribeError {
    fn into_response(self) -> Response {
        match self {
            Self::ValidationError(_) => self.status_code().into_response(),
            Self::UnexpectedError(_) => self.status_code().into_response(),
        }
    }
}

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
) -> Result<StatusCode, SubscribeError> {
    let new_subscriber = form_data.try_into()?;
    let mut transaction = state.db_pool.begin().await.context("Failed to acquire a database connection.")?;
    let subscriber_id = insert_subscriber(&mut transaction, &new_subscriber).await
        .context("Failed to insert new subscriber.")?;
    let subscription_token = generate_subscription_token();
    store_token(&mut transaction, subscriber_id, &subscription_token).await
        .context("Failed to store the confirmation token for a new subscriber")?;
    transaction.commit().await.context("Failed to commit SQL transaction to store a new subscriber.")?;

    send_confirmation_email(
        new_subscriber, 
        &state.email_client, 
        &state.base_url,
        &subscription_token,
    )
    .await
    .context("Failed to send a confirmation email.")?;
    Ok(StatusCode::OK)
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
#[tracing::instrument(name = "Saving new subscriber details in the database", skip(new_subscriber, transaction))]
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
    .await?;
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
    .await?;
    Ok(())
}
