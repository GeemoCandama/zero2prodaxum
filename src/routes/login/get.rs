use axum::{
    extract::FromRef,
    response::Html,
    http::StatusCode,
};
use axum_extra::extract::cookie::{SignedCookieJar, Cookie, Key};
use crate::startup::AppState;

impl FromRef<AppState> for Key {
    fn from_ref(state: &AppState) -> Self {
        state.secret.clone()
    }
}

pub async fn login_form(jar: SignedCookieJar) -> (StatusCode, SignedCookieJar, Html<String>) {
    let error_html = match jar.get("_flash") {
        Some(cookie) => format!(r#"<p><i>{}</i></p>"#, cookie.value()),
        None => String::new(),
    };
    let html = Html(
        format!(r#"<!doctype html>
<html lang="en">
<head>
    <meta http-equiv="content-type" content="text/html; charset=utf-8">
    <title>Login</title>
</head>
<body>
    {error_html}
    <form action="/login" method="post">
        <label>Username
            <input
                type="text"
                placeholder="Enter Username"
                name="username"
            >
        </label>
        <label>Password
            <input
                type="password"
                placeholder="Enter Password"
                name="password"
            >
        </label>
        <button type="submit">Login</button>
    </form>
</body>
</html>"#,)
);
    (
        StatusCode::OK, 
        jar.remove(Cookie::named("_flash")),
        html
    )
}
