use axum::response::{IntoResponse, Response};
use axum_sessions::extractors::ReadableSession;
use axum::{
    extract::State,
    response::Html,
    http::header::LOCATION,
};
use http::StatusCode;
use anyhow::Context;

use crate::startup::AppState;

pub const USER_ID_COOKIE: &str = "user_id";

pub async fn admin_dashboard(
    State(state): State<AppState>,
    session: ReadableSession,
) -> Response {
    let username = if let Some(user_id) = session
        .get::<uuid::Uuid>(USER_ID_COOKIE)
    {
        match get_username(user_id, &state.db_pool).await {
            Ok(username) => username,
            Err(_) => {
                tracing::error!("Failed to get username from database");
                return (
                    StatusCode::INTERNAL_SERVER_ERROR, 
                    "Internal Server Error"
                ).into_response();
            }
        }
    } else {
        return (
            StatusCode::SEE_OTHER,
            [
                (LOCATION, "/login"),
            ],
        ).into_response();
    };
    let html = Html(
        format!(r#"<!doctype html>
<html lang="en">
<head>
    <meta http-equiv="content-type" content="text/html; charset=utf-8">
    <title>Login</title>
</head>
<body>
    <p>Welcome {username}!</p>
    <p>Available actions:</p>
    <ol>
        <li><a href="/admin/password">Change password</a></li>
        <li>
            <form name="logoutForm" action="/admin/logout" method="post">
                <input type="submit" value="Logout">
            </form>
        </li>
    </ol>
</body>
</html>"#,)
);
    html.into_response()
}

#[tracing::instrument(
    name = "Get username",
    skip(pool),
)]
pub async fn get_username(
    user_id: uuid::Uuid,
    pool: &sqlx::PgPool,
) -> Result<String, anyhow::Error> {
    let row = sqlx::query!(
        r#"
        SELECT username
        FROM users
        WHERE user_id = $1
        "#,
        user_id
    )
    .fetch_one(pool)
    .await
    .context("Failed to fetch username")?;
    Ok(row.username)
}
