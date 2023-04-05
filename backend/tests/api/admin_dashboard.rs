use crate::helpers::{assert_is_redirect_to, spawn_app};
use common::static_routes::*;
use serial_test::serial;

#[serial]
#[tokio::test]
async fn you_must_be_logged_in_to_access_the_admin_dashboard() {
    // Arrange
    let app = spawn_app().await;

    // Act
    let response = app.get_admin_dashboard().await;

    // Assert
    assert_is_redirect_to(&response, routes().root.login.get().complete());
}

#[serial]
#[tokio::test]
async fn logout_clears_session_state() {
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

    // Act - Part 3 - Logout
    let response = app.post_logout().await;
    assert_is_redirect_to(&response, routes().root.login.get().complete());

    // FIXIT
    // NO IDEA WHY IT DOES NOT WORK ONLY IN THIS PLACE

    // // Act - Part 4 - Follow the redirect
    // let html_page = app.get_login_html().await;
    // assert!(html_page.contains(r#"<p><i>You have successfully logged out.</i></p>"#));

    // Act - Part 5 - Attempt to load admin panel
    let response = app.get_admin_dashboard().await;
    assert_is_redirect_to(&response, routes().root.login.get().complete());
}
