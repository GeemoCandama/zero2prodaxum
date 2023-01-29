use crate::helpers::{spawn_app, assert_is_redirect_to};

#[tokio::test]
async fn you_must_be_logged_in_to_access_admin_dashboard() {
    let app = spawn_app().await;

    let response = app.get_admin_dashboard().await;
    assert_is_redirect_to(&response, "/login");
}

#[tokio::test]
async fn logout_clears_session_state() {
    let app = spawn_app().await;

    let login_body = serde_json::json!({
        "username": &app.test_user.username,
        "password": &app.test_user.password,
    });
    let response = app.post_login(&login_body).await;
    assert_is_redirect_to(&response, "/admin/dashboard");

    let html_page = app.get_admin_dashboard_html().await;
    assert!(html_page.contains(&app.test_user.username));

    let response = app.post_logout().await;
    assert_is_redirect_to(&response, "/login");

    // let html_page = app.get_login_html().await;
    //  The current approach to cookies and one time messages does
    //  not allow for cross endoint messages. This could be solved by using
    //  axum_flash and more generally storing the messages in the session.
    // assert!(html_page.contains("You have been logged out."));

    let response = app.get_admin_dashboard().await;
    assert_is_redirect_to(&response, "/login");
}
