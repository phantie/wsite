use api_aga_in::configuration::get_configuration;
use api_aga_in::database::*;
use api_aga_in::startup::Application;
use api_aga_in::telemetry::{get_subscriber, init_subscriber};
use once_cell::sync::Lazy;
use std::sync::Arc;
use wiremock::MockServer;

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

pub async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);

    let email_server = MockServer::start().await;

    let configuration = {
        let mut c = get_configuration();
        c.database = c.testing.database.clone();
        c.application = c.testing.application.clone();
        c.email_client.base_url = email_server.uri();
        c
    };

    let application = Application::build(&configuration).await;

    let host = application.host();
    let port = application.port();

    let address = format!("http://{}:{}", host, port);
    let database = application.database();
    add_test_user(database.clone()).await.unwrap();

    let _ = tokio::spawn(application.server());

    TestApp {
        address,
        database,
        email_server,
        port,
    }
}

pub struct TestApp {
    pub address: String,
    pub database: Arc<Database>,
    pub email_server: MockServer,
    pub port: u16,
}

impl TestApp {
    pub async fn post_subscriptions(&self, body: String) -> reqwest::Response {
        reqwest::Client::new()
            .post(&format!("{}/subscriptions", &self.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    /// Extract the confirmation links embedded in the request to the email API.
    pub fn get_confirmation_links(&self, email_request: &wiremock::Request) -> ConfirmationLinks {
        let body: serde_json::Value = serde_json::from_slice(&email_request.body).unwrap();
        // Extract the link from one of the request fields.
        let get_link = |s: &str| {
            let links: Vec<_> = linkify::LinkFinder::new()
                .links(s)
                .filter(|l| *l.kind() == linkify::LinkKind::Url)
                .collect();
            assert_eq!(links.len(), 1);
            let raw_link = links[0].as_str().to_owned();
            let mut confirmation_link = reqwest::Url::parse(&raw_link).unwrap();
            // Let's make sure we don't call random APIs on the web
            assert_eq!(confirmation_link.host_str().unwrap(), "127.0.0.1");
            confirmation_link.set_port(Some(self.port)).unwrap();
            confirmation_link
        };
        let html = get_link(&body["HtmlBody"].as_str().unwrap());
        let plain_text = get_link(&body["TextBody"].as_str().unwrap());
        ConfirmationLinks { html, plain_text }
    }

    pub async fn post_newsletters(&self, body: serde_json::Value) -> reqwest::Response {
        let (username, password) = self.test_user().await;
        reqwest::Client::new()
            .post(&format!("{}/newsletters", &self.address))
            .basic_auth(username, Some(password))
            .json(&body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn test_user(&self) -> (String, String) {
        let user_docs = User::all_async(&self.database.collections.users)
            .await
            .unwrap();

        let user = user_docs.into_iter().next().unwrap();

        (user.contents.username, user.contents.password)
    }
}

pub struct ConfirmationLinks {
    pub html: reqwest::Url,
    pub plain_text: reqwest::Url,
}

async fn add_test_user(
    database: Arc<Database>,
) -> Result<CollectionDocument<User>, bonsaidb::core::schema::InsertError<User>> {
    User {
        username: uuid::Uuid::new_v4().to_string(),
        password: uuid::Uuid::new_v4().to_string(),
    }
    .push_into_async(&database.collections.users)
    .await
}
