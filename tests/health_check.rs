use api_aga_in::configuration::get_configuration;
use api_aga_in::database::*;
use api_aga_in::email_client::EmailClient;
use api_aga_in::startup::run;
use api_aga_in::telemetry::{get_subscriber, init_subscriber};
use hyper::StatusCode;
use once_cell::sync::Lazy;
use std::sync::Arc;
// serial needed to run tests that spawn app
// because only one handle to database can be held at a time,
// otherwise the second such test would hang for almost a minute and then panic
//
// either to use one database at a time
// or create several databases to acesss them concurrently
//
// I selected the first approach because the neccesity
// to generate paths and create repositories for databases dissappears
//
// NOTE such tests hang if ran without #[serial]
use serial_test::serial;

static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();

    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    };
});

struct TestApp {
    address: String,
    database: Arc<Database>,
}

async fn test_storage() -> AsyncStorage {
    let configuration = get_configuration();
    storage(&configuration.testing.database.dir, true).await
}

async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);

    let configuration = get_configuration();

    let sender_email = configuration
        .email_client
        .sender()
        .expect("Invalid sender email address.");
    let email_client = Arc::new(EmailClient::new(
        configuration.email_client.base_url,
        sender_email,
        configuration.email_client.authorization_token,
    ));

    // trying to bind port 0 will trigger an OS scan for an available port
    let listener =
        std::net::TcpListener::bind("127.0.0.1:0").expect("Failed to bind free random port");
    let port = listener.local_addr().unwrap().port();

    let storage = test_storage().await;

    let storage = Arc::new(storage);

    let database = Arc::new(Database::init(storage.clone()).await);

    let server = run(listener, database.clone(), email_client);

    let _ = tokio::spawn(server);
    let address = format!("http://127.0.0.1:{}", port);

    TestApp {
        address,
        database: database.clone(),
    }
}

#[serial]
#[tokio::test]
async fn health_check_works() {
    let app = spawn_app().await;

    let client = reqwest::Client::new();
    let response = client
        .get(&format!("{}/health_check", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(StatusCode::OK, response.status());
    assert_eq!(Some(0), response.content_length());
}

#[serial]
#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let response = client
        .post(&format!("{}/subscriptions", &app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");
    assert_eq!(StatusCode::OK, response.status());

    let subscriptions_docs = Subscription::all_async(&app.database.collections.subscriptions)
        .await
        .unwrap();

    assert_eq!(subscriptions_docs.iter().count(), 1);

    // verify the fields of the saved entry
    let Subscription { name, email } = &subscriptions_docs.iter().next().unwrap().contents;

    assert_eq!(name, "le guin");
    assert_eq!(email, "ursula_le_guin@gmail.com");
}

#[serial]
#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];
    for (invalid_body, error_message) in test_cases {
        let response = client
            .post(&format!("{}/subscriptions", &app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request.");

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

// TODO make the test fail and then fix it
#[serial]
#[tokio::test]
async fn subscribe_returns_a_200_when_fields_are_present_but_empty() {
    // Arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=&email=ursula_le_guin%40gmail.com", "empty name"),
        ("name=Ursula&email=", "empty email"),
        ("name=Ursula&email=definitely-not-an-email", "invalid email"),
    ];
    for (body, description) in test_cases {
        // Act
        let response = client
            .post(&format!("{}/subscriptions", &app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request.");
        // Assert
        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not return a 400 when the payload was {}.",
            description
        );
    }
}
