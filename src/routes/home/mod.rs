use axum::{
    response::Html,
    http::StatusCode,
};

pub async fn home() -> (StatusCode, Html<&'static str>) {
    (StatusCode::OK, Html(include_str!("home.html")))
}
