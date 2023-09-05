use backend::configuration;
use backend::startup::Application;
use backend::telemetry;
use hyper::StatusCode;
use once_cell::sync::Lazy;
use reqwest::{RequestBuilder, Response};
use static_routes::*;
use uuid::Uuid;

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

    let env_conf = configuration::EnvConf {
        host: "localhost".into(),
        port: 0,
        session_secret: hex::encode([0_u8; 64]),

        features: configuration::EnvFeatures {},
    };

    let conf = configuration::Conf { env: env_conf };

    let application = Application::build(&conf).await;

    let host = application.host();
    let port = application.port();

    let address = format!("http://{}:{}", host, port);

    let db = application.db();
    let _ = tokio::spawn(application.server());

    let test_user = TestUser::generate();
    test_user.replace(db.clone()).await.unwrap();

    let api_client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .cookie_store(true)
        .build()
        .unwrap();

    TestApp {
        address,
        port,
        test_user,
        api_client,
        db,
    }
}

pub struct TestApp {
    pub address: String,
    pub port: u16,
    pub test_user: TestUser,
    pub api_client: reqwest::Client,
    pub db: cozo::DbInstance,
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

pub struct TestUser {
    pub username: String,
    pub password: String,
}

impl TestUser {
    pub fn generate() -> Self {
        Self {
            username: Uuid::new_v4().to_string(),
            password: Uuid::new_v4().to_string(),
        }
    }

    async fn replace(&self, db: cozo::DbInstance) -> backend::db::OpResult {
        let pwd_hash = auth::hash_pwd(self.password.as_bytes()).unwrap();
        backend::db::q::put_user(&db, &self.username, &pwd_hash)
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

    // cargo test ::test_user_created -- --nocapture

    #[tokio::test]
    async fn test_user_created() {
        // Arrange
        let app = spawn_app().await;
        let user = backend::db::q::find_user_by_username(&app.db, &app.test_user.username).unwrap();
        assert!(user.is_some());
    }

    #[tokio::test]
    async fn users_usernames_must_be_unique() {
        "it's a PK now";
    }
}
