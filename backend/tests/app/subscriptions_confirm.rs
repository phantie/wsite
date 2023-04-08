use crate::helpers::spawn_app;
use api_aga_in::database::*;
use hyper::StatusCode;
use serial_test::serial;
use static_routes::*;
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};

#[serial]
#[tokio::test]
async fn confirmations_without_token_are_rejected_with_a_400() {
    // Arrange
    let app = spawn_app().await;

    // Act
    let response = app.get(routes().api.subs.confirm).send().await.unwrap();

    // Assert
    assert_eq!(response.status().as_u16(), 400);
}

#[serial]
#[tokio::test]
async fn the_link_returned_by_subscribe_returns_a_200_if_called() {
    // Arrange
    let app = spawn_app().await;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;
    app.post_subscriptions(body.into()).await;
    let email_request = &app.email_server.received_requests().await.unwrap()[0];
    let confirmation_links = app.get_confirmation_links(&email_request);

    // Act
    let response = reqwest::get(confirmation_links.html).await.unwrap();

    // Assert
    assert_eq!(StatusCode::OK, response.status());
}

#[serial]
#[tokio::test]
async fn clicking_on_the_confirmation_link_confirms_a_subscriber() {
    // Arrange
    let app = spawn_app().await;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;
    app.post_subscriptions(body.into()).await;
    let email_request = &app.email_server.received_requests().await.unwrap()[0];
    let confirmation_links = app.get_confirmation_links(&email_request);

    // Act
    reqwest::get(confirmation_links.html)
        .await
        .unwrap()
        .error_for_status()
        .unwrap();

    // Assert
    let subscriptions_docs = Subscription::all_async(&app.database.collections.subscriptions)
        .await
        .unwrap();
    assert_eq!(subscriptions_docs.iter().count(), 1);

    let subscription = &subscriptions_docs.iter().next().unwrap().contents;
    assert_eq!(subscription.name, "le guin");
    assert_eq!(subscription.email.as_ref(), "ursula_le_guin@gmail.com");
    assert_eq!(subscription.status, "confirmed");
}
