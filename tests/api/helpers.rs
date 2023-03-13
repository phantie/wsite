use api_aga_in::configuration::get_configuration;
use api_aga_in::database::*;
use api_aga_in::email_client::EmailClient;
use api_aga_in::startup::run;
use api_aga_in::telemetry::{get_subscriber, init_subscriber};
use once_cell::sync::Lazy;
use std::sync::Arc;

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

pub struct TestApp {
    pub address: String,
    pub database: Arc<Database>,
}

async fn test_storage() -> AsyncStorage {
    let configuration = get_configuration();
    storage(&configuration.testing.database.dir, true).await
}

pub async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);

    let configuration = get_configuration();

    let sender_email = configuration
        .email_client
        .sender()
        .expect("Invalid sender email address.");
    let timeout = configuration.email_client.timeout();
    let email_client = Arc::new(EmailClient::new(
        configuration.email_client.base_url,
        sender_email,
        configuration.email_client.authorization_token,
        timeout,
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
