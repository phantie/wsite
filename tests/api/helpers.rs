use api_aga_in::configuration::get_configuration;
use api_aga_in::database::*;
use api_aga_in::startup::Application;
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

pub async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);

    let configuration = {
        let mut c = get_configuration();
        c.database = c.testing.database.clone();
        c.application = c.testing.application.clone();
        c
    };

    let application = Application::build(&configuration).await;

    let address = format!("http://{}:{}", application.host(), application.port());
    let database = application.database();

    let _ = tokio::spawn(application.server());

    TestApp { address, database }
}
