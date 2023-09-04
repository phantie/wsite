use backend::configuration;
pub use backend::database::*;
use backend::startup::Application;
use backend::telemetry;
use bonsaidb::server::BonsaiListenConfig;
use hyper::StatusCode;
use once_cell::sync::Lazy;
use reqwest::{RequestBuilder, Response};
use static_routes::*;
use std::net::UdpSocket;
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

    let (db_server, db_storage_location) = db::init::test_server().await.unwrap();

    let db_port = UdpSocket::bind("localhost:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port();

    let listen_config = BonsaiListenConfig::default()
        .port(db_port)
        .reuse_address(true);

    let cert = db_server
        .certificate_chain()
        .await
        .unwrap()
        .into_end_entity_certificate();

    let _ = tokio::spawn(async move {
        db_server
            .listen_on(listen_config)
            .await
            .expect("failed to start db server");
    });

    let env_conf = configuration::EnvConf {
        host: "localhost".into(),
        port: 0,
        session_secret: hex::encode([0_u8; 64]),
        db: configuration::EnvDbClientConf {
            host: "localhost".into(),
            password: None,
            port: db_port,
        },

        features: configuration::EnvFeatures {},
    };

    let conf = configuration::Conf {
        db_client: configuration::DbClientConf::Testing {
            quic_url: format!("bonsaidb://{}:{}", env_conf.db.host, env_conf.db.port),
            cert,
        },
        env: env_conf,
    };

    let application = Application::build(&conf).await;

    let host = application.host();
    let port = application.port();

    let address = format!("http://{}:{}", host, port);

    let db_client = application.db_client.clone();

    let _ = tokio::spawn(application.server());

    let test_user = TestUser::generate();
    test_user.store(db_client.clone()).await.unwrap();

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
        db_client,
        _db_storage_location: db_storage_location,
    }
}

pub struct TestApp {
    pub address: String,
    pub port: u16,
    pub test_user: TestUser,
    pub api_client: reqwest::Client,
    pub db_client: SharedDbClient,
    // drop last with app
    _db_storage_location: tempdir::TempDir,
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
        db_client: SharedDbClient,
    ) -> Result<CollectionDocument<schema::User>, bonsaidb::core::schema::InsertError<schema::User>>
    {
        let password_hash = auth::hash_pwd(self.password.as_bytes()).unwrap();

        schema::User {
            username: self.username.clone(),
            password_hash,
        }
        .push_into_async(&db_client.read().await.collections().users)
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

    // cargo test ::test_user_created -- --nocapture

    #[tokio::test]
    async fn test_user_created() {
        // Arrange
        let app = spawn_app().await;
        let db_client = app.db_client.read().await.client();
        let users = db_client.database::<schema::User>("users").await.unwrap();
        let user_docs = schema::User::all_async(&users).await.unwrap();
        assert_eq!(user_docs.len(), 1);
    }

    #[tokio::test]
    async fn users_usernames_must_be_unique() {
        // Arrange
        let app = spawn_app().await;

        let test_user = &app.test_user;

        let ok = match test_user.store(app.db_client.clone()).await {
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
