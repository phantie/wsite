use crate::helpers::{assert_is_redirect_to, spawn_app};
use api_aga_in::static_routes::*;
#[allow(unused_imports)]
use hyper::StatusCode;
use serial_test::serial;

#[serial]
#[tokio::test]
async fn redirect_to_admin_dashboard_after_login_success() {
    // Arrange
    let app = spawn_app().await;

    // Act - Part 1 - Login
    let login_body = serde_json::json!({
        "username": &app.test_user.username,
        "password": &app.test_user.password
    });
    let response = app.post_login(&login_body).await;
    assert_is_redirect_to(&response, routes().root.admin.dashboard.get().complete());

    // Act - Part 2 - Follow the redirect
    let html_page = app.get_admin_dashboard_html().await;
    assert!(html_page.contains(&format!("Welcome {}", app.test_user.username)));

    // Act - Part 3 - Renew session and go to dashboard again
    let _response = app.post_login(&login_body).await;
    let html_page = app.get_admin_dashboard_html().await;
    assert!(html_page.contains(&format!("Welcome {}", app.test_user.username)));
}
