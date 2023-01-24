use axum::extract::{Query, State};
use crate::startup::AppState;
use http::StatusCode;

#[derive(serde::Deserialize, Debug)]
pub struct Parameters {
    subscription_token: String,
}

#[tracing::instrument(
    name = "Confirm a pending subscriber",
    skip(state, parameters),
)]
pub async fn confirm(
    State(state): State<AppState>,
    parameters: Query<Parameters>,
) -> StatusCode {
    let id = match get_subscriber_id_from_token(&state.db_pool, &parameters.subscription_token).await {
        Ok(id) => id,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR,
    };
    match id {
        Some(id) => {
            if confirm_subscriber(&state.db_pool, id).await.is_err() {
                return StatusCode::INTERNAL_SERVER_ERROR;
            }
            StatusCode::OK
        }
        None => StatusCode::UNAUTHORIZED,
    }
}

#[tracing::instrument(
    name = "Get a subscriber id from a subscription token",
    skip(pool, subscription_token),
)]
pub async fn get_subscriber_id_from_token(
    pool: &sqlx::PgPool,
    subscription_token: &str,
) -> Result<Option<uuid::Uuid>, sqlx::Error> {
    let result = sqlx::query!(
        r#"
        SELECT subscriber_id
        FROM subscription_tokens
        WHERE subscription_token = $1
        "#,
        subscription_token
    )
    .fetch_optional(pool)
    .await
    .map_err(|error| {
        tracing::error!("Failed to execute query: {:?}", error);
        error
    })?;
    Ok(result.map(|r|r.subscriber_id))
}

#[tracing::instrument(
    name = "Confirm a subscriber",
    skip(pool, subscriber_id),
)]
pub async fn confirm_subscriber(
    pool: &sqlx::PgPool,
    subscriber_id: uuid::Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        UPDATE subscriptions
        SET status = 'confirmed'
        WHERE id = $1
        "#,
        subscriber_id
    )
    .execute(pool)
    .await
    .map_err(|error| {
        tracing::error!("Failed to execute query: {:?}", error);
        error
    })?;
    Ok(())
}
