use axum_extra::extract::cookie::{SignedCookieJar, Cookie};
use axum_sessions::extractors::WritableSession;
use http::header::LOCATION;
use secrecy::{Secret, ExposeSecret};
use axum::{
    extract::{Form, State},
    response::{Response, IntoResponse, Redirect},
    http::StatusCode,
};

use crate::{
    routes::admin::dashboard::{USER_ID_COOKIE, get_username}, 
    startup::AppState, authentication::{validate_credentials, AuthError, Credentials}
};

#[derive(serde::Deserialize)]
pub struct FormData {
    current_password: Secret<String>,
    new_password: Secret<String>,
    new_password_check: Secret<String>,
}

pub async fn change_password(
    session: WritableSession,
    jar: SignedCookieJar,
    State(state): State<AppState>,
    Form(form): Form<FormData>,
) -> Response {
    let user_id = session.get::<uuid::Uuid>(USER_ID_COOKIE);
    if user_id.is_none() {
        return Redirect::to("/login").into_response();
    };
    let user_id = user_id.unwrap();
    if form.new_password.expose_secret() != form.new_password_check.expose_secret() {
        return change_password_error_response(
           "You entered two different new passwords - the field values must match.".to_string(),
            jar
        );
    };
    let username = match get_username(user_id, &state.db_pool).await {
        Err(_) => return change_password_error_response(
            "Failed to get username from database".to_string(),
            jar
        ),
        Ok(username) => username,
    };
    let credentials = Credentials {
        username,
        password: form.current_password,
    };
    if let Err(e) = validate_credentials(credentials, &state.db_pool).await {
        return match e {
            AuthError::InvalidCredentials(_) => {
                change_password_error_response(
                    "The current password is incorrect.".to_string(),
                    jar
                )
            },
            AuthError::UnexpectedError(_) => {
                change_password_error_response(
                    "An unexpected error occurred.".to_string(),
                    jar
                )
            },
        }
    };
    if new_password_is_too_short(&form.new_password) {
        return change_password_error_response(
            "The new password is too short - 12 character minimum".to_string(),
            jar
        );
    };
    if new_password_is_too_long(&form.new_password) {
        return change_password_error_response(
            "The new password is too long 128 character maximum".to_string(),
            jar
        );
    };
    match crate::authentication::change_password
        (
            user_id, 
            form.new_password, 
            &state.db_pool
        )
        .await
    {
        Err(_) => change_password_error_response(
            "Failed to change password".to_string(),
            jar
        ),
        Ok(_) => (
            StatusCode::SEE_OTHER,
            [
                (LOCATION, "/admin/password"),
            ],
            jar.add(Cookie::new("cperror", "Your password has been changed.".to_string())),
        ).into_response(),
    }
}

fn new_password_is_too_short(password: &Secret<String>) -> bool {
    password.expose_secret().len() < 12
}

fn new_password_is_too_long(password: &Secret<String>) -> bool {
    password.expose_secret().len() > 128
}

fn change_password_error_response(error_string: String, jar: SignedCookieJar) -> Response {
    (
        StatusCode::SEE_OTHER,
        [
            (LOCATION, "/admin/password"),
        ],
        jar.add(Cookie::new("cperror", error_string)),
    ).into_response()
}
