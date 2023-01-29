// Consider using a typed session to enable a more type-safe API if the application
// is more complicated.
// For this project we will use the interface provided by axum_sessions. That is:
// WritableSession and ReadableSession.

use axum::{
    extract::{State, Form},
    response::{IntoResponse, Response},
    http::{
        header::LOCATION,
        StatusCode,
    },
};
use axum_sessions::extractors::WritableSession;
use secrecy::Secret;
use axum_extra::extract::cookie::{SignedCookieJar, Cookie};

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
    jar: SignedCookieJar,
    session: WritableSession,
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
            match insert_user_id(session, user_id) {
                Ok(_) => {
                    tracing::Span::current().record("user_id", &tracing::field::display(&user_id));
                    (
                        StatusCode::SEE_OTHER,
                        [
                            (LOCATION, "/admin/dashboard"),
                        ],
                    ).into_response()
                },
                Err(e) => {
                    tracing::error!("Failed to insert user_id into session");
                    login_error_response(e.to_string(), jar)
                }
            }
        },
        Err(e) => {
            let e = match e {
                AuthError::InvalidCredentials(_) => LoginError::AuthError(e.into()),
                AuthError::UnexpectedError(_) => LoginError::UnexpectedError(e.into()),
            };
            login_error_response(e.to_string(), jar)
        },
    }
}

pub fn insert_user_id(
    mut session: WritableSession,
    user_id: uuid::Uuid,
) -> Result<(), serde_json::Error> {
    session.regenerate();
    session.insert("user_id", user_id)
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

fn login_error_response(error_string: String, jar: SignedCookieJar) -> Response {
    (
        StatusCode::SEE_OTHER,
        [
            (LOCATION, "/login"),
        ],
        jar.add(Cookie::new("_flash", error_string)),
    ).into_response()
}
