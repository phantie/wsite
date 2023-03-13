use api_aga_in::configuration::get_configuration;
use api_aga_in::database::*;
use api_aga_in::email_client::EmailClient;
use api_aga_in::startup::run;
use api_aga_in::telemetry::{get_subscriber, init_subscriber};
use std::sync::Arc;

#[tokio::main]
async fn main() -> hyper::Result<()> {
    let subscriber = get_subscriber("api_aga_in".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

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

    let address = format!(
        "{}:{}",
        configuration.application.host, configuration.application.port
    );
    let listener = std::net::TcpListener::bind(&address).unwrap();
    tracing::info!("Listening on {}", address);

    let storage = Arc::new(storage(&configuration.database.dir, false).await);
    let database = Arc::new(Database::init(storage.clone()).await);
    run(listener, database, email_client).await
}
