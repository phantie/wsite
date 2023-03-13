use crate::helpers::spawn_app;
use api_aga_in::database::*;
use hyper::StatusCode;
use serial_test::serial;

#[serial]
#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    let app = spawn_app().await;
    let response = app.post_subscriptions(body.into()).await;
    assert_eq!(StatusCode::OK, response.status());

    let subscriptions_docs = Subscription::all_async(&app.database.collections.subscriptions)
        .await
        .unwrap();

    assert_eq!(subscriptions_docs.iter().count(), 1);

    // verify the fields of the saved entry
    let subscription = &subscriptions_docs.iter().next().unwrap().contents;

    assert_eq!(subscription.name, "le guin");
    assert_eq!(subscription.email, "ursula_le_guin@gmail.com");
}

#[serial]
#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    let app = spawn_app().await;
    for (invalid_body, error_message) in test_cases {
        let response = app.post_subscriptions(invalid_body.into()).await;

        assert_eq!(
            StatusCode::UNPROCESSABLE_ENTITY,
            response.status(),
            // Additional customised error message on test failure
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_message
        );
    }

    let subscriptions_docs = Subscription::all_async(&app.database.collections.subscriptions)
        .await
        .unwrap();

    assert_eq!(subscriptions_docs.iter().count(), 0);
}

#[serial]
#[tokio::test]
async fn subscribe_returns_a_200_when_fields_are_present_but_empty() {
    let test_cases = vec![
        ("name=&email=ursula_le_guin%40gmail.com", "empty name"),
        ("name=Ursula&email=", "empty email"),
        ("name=Ursula&email=definitely-not-an-email", "invalid email"),
    ];

    let app = spawn_app().await;
    for (body, description) in test_cases {
        let response = app.post_subscriptions(body.into()).await;

        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not return a 400 when the payload was {}.",
            description
        );
    }
}
