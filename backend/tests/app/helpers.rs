use api_aga_in::configuration::get_configuration;
use api_aga_in::database::*;
use api_aga_in::startup::Application;
use api_aga_in::telemetry;
use argon2::{password_hash::SaltString, Algorithm, Argon2, Params, PasswordHasher, Version};
use hyper::StatusCode;
use once_cell::sync::Lazy;
use reqwest::{RequestBuilder, Response};
use static_routes::*;
use std::sync::Arc;
use uuid::Uuid;
use wiremock::MockServer;

static TRACING: Lazy<()> = Lazy::new(|| {
    let subscriber = telemetry::TracingSubscriber::new("testing");

    if std::env::var("TEST_LOG").is_ok() {
        telemetry::init_global_default(subscriber.build(std::io::stdout));
    } else {
        telemetry::init_global_default(subscriber.build(std::io::sink));
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

    let _ = tokio::spawn(application.server());

    let test_user = TestUser::generate();
    test_user.store(database.clone()).await.unwrap();

    let api_client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .cookie_store(true)
        .build()
        .unwrap();

    TestApp {
        address,
        database,
        email_server,
        port,
        test_user,
        api_client,
    }
}

pub struct TestApp {
    pub address: String,
    pub database: Arc<Database>,
    pub email_server: MockServer,
    pub port: u16,
    pub test_user: TestUser,
    pub api_client: reqwest::Client,
}

impl TestApp {
    pub fn get(&self, static_path: impl Get) -> RequestBuilder {
        self.api_client
            .get(static_path.get().with_base(&self.address).complete())
    }

    pub fn post(&self, static_path: impl Post) -> RequestBuilder {
        self.api_client
            .post(static_path.post().with_base(&self.address).complete())
    }

    pub async fn post_subscriptions(&self, body: String) -> reqwest::Response {
        self.post(routes().api.subs.new)
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
        self.post(routes().api.newsletters)
            .basic_auth(&self.test_user.username, Some(&self.test_user.password))
            .json(&body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn post_login<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.post(routes().api.login)
            .form(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn get_home(&self) -> Response {
        self.get(routes().root.home)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    #[allow(dead_code)]
    pub async fn get_login(&self) -> Response {
        self.get(routes().root.login)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn get_admin_dashboard(&self) -> reqwest::Response {
        self.get(routes().root.admin.dashboard)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn get_admin_dashboard_html(&self) -> String {
        self.get_admin_dashboard().await.text().await.unwrap()
    }

    pub async fn get_change_password(&self) -> reqwest::Response {
        self.get(routes().root.admin.password)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn post_change_password<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.post(routes().api.admin.password)
            .form(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn get_change_password_html(&self) -> String {
        self.get_change_password().await.text().await.unwrap()
    }

    pub async fn post_logout(&self) -> reqwest::Response {
        self.post(routes().api.admin.logout)
            .send()
            .await
            .expect("Failed to execute request.")
    }
}

pub struct ConfirmationLinks {
    pub html: reqwest::Url,
    pub plain_text: reqwest::Url,
}

pub struct TestUser {
    pub user_id: Uuid,
    pub username: String,
    pub password: String,
}

impl TestUser {
    pub fn generate() -> Self {
        Self {
            user_id: Uuid::new_v4(),
            username: Uuid::new_v4().to_string(),
            password: Uuid::new_v4().to_string(),
        }
    }

    async fn store(
        &self,
        database: Arc<Database>,
    ) -> Result<CollectionDocument<User>, bonsaidb::core::schema::InsertError<User>> {
        let salt = SaltString::generate(&mut rand::thread_rng());
        let password_hash = Argon2::new(
            Algorithm::Argon2id,
            Version::V0x13,
            Params::new(15000, 1, 1, None).unwrap(),
        )
        .hash_password(self.password.as_bytes(), &salt)
        .unwrap()
        .to_string();

        User {
            username: self.username.clone(),
            password_hash: password_hash,
        }
        .push_into_async(&database.collections.users)
        .await
    }
}

pub fn assert_is_redirect_to(response: &reqwest::Response, location: impl Get) {
    assert_eq!(StatusCode::SEE_OTHER, response.status());
    assert_eq!(
        location.get().complete(),
        response.headers().get("Location").unwrap()
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[serial]
    #[tokio::test]
    async fn users_usernames_must_be_unique() {
        // Arrange
        let app = spawn_app().await;

        let test_user = &app.test_user;

        let ok = match test_user.store(app.database.clone()).await {
            Ok(_) => false,
            Err(e) => match e.error {
                bonsaidb::core::Error::UniqueKeyViolation { .. } => true,
                _ => false,
            },
        };

        // Assert
        if !ok {
            panic!("inserting the same user should emit UniqueKeyViolation on username field");
        }
    }
}
