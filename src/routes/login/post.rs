use axum::{
    extract::{State, Form},
    response::{Redirect, IntoResponse, Response},
    http::{
        header::{SET_COOKIE, LOCATION},
        StatusCode,
    },
};
use secrecy::Secret;

use crate::authentication::{validate_credentials, Credentials, AuthError};
use crate::startup::AppState;
use crate::routes::error_chain_fmt;

#[derive(serde::Deserialize, Debug)]
pub struct FormData {
    pub username: String,
    pub password: Secret<String>,
}

#[tracing::instrument(
    skip(login_form, state),
    fields(
        username=tracing::field::Empty,
        user_id=tracing::field::Empty,
    )
)]
pub async fn login(
    State(state): State<AppState>,
    Form(login_form): Form<FormData>
) -> Response {
    let credentials = Credentials {
        username: login_form.username,
        password: login_form.password,
    };
    tracing::Span::current()
        .record("username", &tracing::field::debug(&credentials.username));
    match validate_credentials(credentials, &state.db_pool).await {
        Ok(user_id) => {
            tracing::Span::current().record("user_id", &tracing::field::display(&user_id));
            Redirect::to("/").into_response()
        },
        Err(e) => {
            let e = match e {
                AuthError::InvalidCredentials(_) => LoginError::AuthError(e.into()),
                AuthError::UnexpectedError(_) => LoginError::UnexpectedError(e.into()),
            };
            (
                StatusCode::SEE_OTHER,
                [
                    (LOCATION, "/login"),
                    (SET_COOKIE, &format!("_flash={e}")),
                ],
            ).into_response()
        },
    }
}

#[derive(thiserror::Error)]
pub enum LoginError {
    #[error("Authentication failed")]
    AuthError(#[source] anyhow::Error),
    #[error("Something went wrong")]
    UnexpectedError(#[from] anyhow::Error),
}

impl std::fmt::Debug for LoginError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}
