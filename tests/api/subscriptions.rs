use crate::helpers::spawn_app;
use api_aga_in::database::*;
use hyper::StatusCode;
use serial_test::serial;
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};

#[serial]
#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    // Arrange
    let app = spawn_app().await;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    let response = app.post_subscriptions(body.into()).await;

    assert_eq!(StatusCode::OK, response.status());
}

#[serial]
#[tokio::test]
async fn subscribe_persists_the_new_subscriber() {
    // Arrange
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let app = spawn_app().await;
    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    // Act
    let _response = app.post_subscriptions(body.into()).await;

    // Assert
    let subscriptions_docs = Subscription::all_async(&app.database.collections.subscriptions)
        .await
        .unwrap();
    assert_eq!(subscriptions_docs.iter().count(), 1);

    let subscription = &subscriptions_docs.iter().next().unwrap().contents;
    assert_eq!(subscription.name, "le guin");
    assert_eq!(subscription.email.as_ref(), "ursula_le_guin@gmail.com");
    assert_eq!(subscription.status, "pending_confirmation");
}

#[serial]
#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    // Arrange
    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];
    let app = spawn_app().await;

    // Act
    for (invalid_body, error_message) in test_cases {
        let response = app.post_subscriptions(invalid_body.into()).await;

        assert_eq!(
            StatusCode::UNPROCESSABLE_ENTITY,
            response.status(),
            "The API did not fail with 422 Unprocessable Content when the payload was {}.",
            error_message
        );
    }

    // Assert
    let subscriptions_docs = Subscription::all_async(&app.database.collections.subscriptions)
        .await
        .unwrap();

    assert_eq!(subscriptions_docs.iter().count(), 0);
}

#[serial]
#[tokio::test]
async fn subscribe_returns_a_200_when_fields_are_present_but_empty() {
    // Arrange
    let test_cases = vec![
        ("name=&email=ursula_le_guin%40gmail.com", "empty name"),
        ("name=Ursula&email=", "empty email"),
        ("name=Ursula&email=definitely-not-an-email", "invalid email"),
    ];
    let app = spawn_app().await;

    // Act
    for (body, description) in test_cases {
        let response = app.post_subscriptions(body.into()).await;
        // Assert
        assert_eq!(
            StatusCode::BAD_REQUEST,
            response.status(),
            "The API did not return a 400 Bad Request when the payload was {}.",
            description
        );
    }
}

#[serial]
#[tokio::test]
async fn subscribe_sends_a_confirmation_email_for_valid_data() {
    // Arrange
    let app = spawn_app().await;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    // Act
    app.post_subscriptions(body.into()).await;

    // Assert on mock drop
}

#[serial]
#[tokio::test]
async fn subscribe_sends_a_confirmation_email_with_a_link() {
    // Arrange
    let app = spawn_app().await;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    // Act
    app.post_subscriptions(body.into()).await;

    // Assert
    // Get the first intercepted request
    let email_request = &app.email_server.received_requests().await.unwrap()[0];
    let confirmation_links = app.get_confirmation_links(&email_request);
    assert_eq!(confirmation_links.html, confirmation_links.plain_text);
}
