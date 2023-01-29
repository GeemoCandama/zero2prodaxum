use axum::response::{IntoResponse, Redirect, Response};
use axum_extra::extract::{cookie::Cookie, SignedCookieJar};
use axum_sessions::extractors::WritableSession;
use http::{header::LOCATION, StatusCode};

use super::dashboard::USER_ID_COOKIE;

pub async fn logout(mut session: WritableSession, jar: SignedCookieJar) -> Response {
    if session.get::<uuid::Uuid>(USER_ID_COOKIE).is_none() {
        Redirect::to("/login").into_response()
    } else {
        session.destroy();
        (
            StatusCode::SEE_OTHER,
            [
                (LOCATION, "/login"),
            ],
            jar.add(Cookie::new("_flash", "You have been logged out.".to_string())),
        )
        .into_response()
    }
}
