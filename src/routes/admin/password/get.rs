use axum::{
    http::StatusCode,
    response::{Html, Response, IntoResponse, Redirect},
};
use axum_extra::extract::SignedCookieJar;
use axum_sessions::extractors::ReadableSession;

use crate::routes::admin::dashboard::USER_ID_COOKIE;

pub async fn change_password_form(
    session: ReadableSession,
    jar: SignedCookieJar,
) -> Response {
    if session.get::<uuid::Uuid>(USER_ID_COOKIE).is_none() {
        return Redirect::to("/login").into_response();
    }
    let error_html = match jar.get("cperror") {
        Some(cookie) => format!(r#"<p><i>{}</i></p>"#, cookie.value()),
        None => String::new(),
    };

    let html = Html(format!(r#"<!DOCTYPE html>
    <html lang="en">
    <head>
        <meta http-equiv="content-type" content="text/html; charset=utf-8">
        <title>Change Password</title>
    </head>
    <body>
        {error_html}
        <form action="/admin/password" method="post">
            <label>Current password
                <input
                    type="password"
                    placeholder="Enter current password"
                    name="current_password"
                >
            </label>
            <br>
            <label>New password
                <input
                    type="password"
                    placeholder="Enter new password"
                    name="new_password"
                >
            </label>
            <br>
            <label>Confirm new password
                <input
                    type="password"
                    placeholder="Type the new password again"
                    name="new_password_check"
                >
            </label>
            <br>
            <button type="submit">Change password</button>
        </form>
        <p><a href="/admin/dashboard">&lt;- Back</a></p>
    </body>
    </html>"#,));
    (StatusCode::OK, html).into_response()
}
