use axum::{
    http::StatusCode,
    routing::{get, post},
    Extension, Json, Router,
};
use bonsaidb::{
    core::{
        connection::{AsyncConnection, AsyncStorageConnection},
        schema::SerializedCollection,
    },
    server::CustomServer,
};
use secrecy::ExposeSecret;
use std::sync::Arc;
use tokio::task::JoinHandle;
use tower_http::add_extension::AddExtensionLayer;
mod database;
use database_common::schema;

struct HostedDatabase {
    server: CustomServer,
    handle: JoinHandle<Result<(), bonsaidb::server::Error>>,
}

impl HostedDatabase {
    fn running(&self) -> bool {
        !self.handle.is_finished()
    }
}

type SharedHostedDatabase = Arc<HostedDatabase>;

#[tokio::main]
async fn main() -> hyper::Result<()> {
    let database_server = database::server().await.unwrap();

    let handle = {
        let database_server = database_server.clone();
        tokio::spawn(async move {
            println!("database server is listening on 5645");
            database_server.listen_on(5645).await
        })
    };

    let hosted_database = Arc::new(HostedDatabase {
        server: database_server,
        handle,
    });

    let app = Router::new()
        .route("/", get(root))
        .route("/health", get(health_check))
        .route("/database/info", get(database_info))
        .route("/users/", post(create_user))
        .layer(AddExtensionLayer::new(hosted_database));

    let addr = database_common::ADDR;

    let listener = std::net::TcpListener::bind(addr).unwrap();

    println!("http server listening on {addr}");

    axum::Server::from_tcp(listener)
        .unwrap()
        .serve(app.into_make_service())
        .await
}

// basic handler that responds with a static string
async fn root() -> &'static str {
    "Hello, World!"
}

#[axum_macros::debug_handler]
async fn database_info(
    Extension(database_server): Extension<SharedHostedDatabase>,
) -> Json<interfacing::DatabaseInfo> {
    Json::from(interfacing::DatabaseInfo {
        is_running: database_server.running(),
    })
}

#[axum_macros::debug_handler]
async fn create_user(
    Extension(database_server): Extension<SharedHostedDatabase>,
    Json(form): Json<interfacing::LoginForm>,
) -> String {
    use argon2::{password_hash::SaltString, Algorithm, Argon2, Params, PasswordHasher, Version};
    let users = database_server
        .server
        .database::<schema::User>("users")
        .await
        .unwrap();

    let username = form.username;
    let password = form.password.expose_secret();

    let user = users
        .view::<schema::UserByUsername>()
        .with_key(username.clone())
        .query_with_collection_docs()
        .await
        .unwrap();
    let user = user.into_iter().next();

    let salt = SaltString::generate(&mut rand::thread_rng());
    let password_hash = Argon2::new(
        Algorithm::Argon2id,
        Version::V0x13,
        Params::new(15000, 2, 1, None).unwrap(),
    )
    .hash_password(password.as_bytes(), &salt)
    .unwrap()
    .to_string();

    match user {
        None => {
            schema::User {
                username: username.clone(),
                password_hash: password_hash,
            }
            .push_into_async(&users)
            .await
            .unwrap();
            "User has been created".into()
        }
        Some(user) => {
            let current_password_hash = user.document.contents.password_hash.as_str();
            // dbg!(&password_hash);
            // dbg!(&current_password_hash);
            if password_hash == current_password_hash {
                // TODO copare hashes correctly
                "User exists, password has NOT been updated".into()
            } else {
                let mut user = user.document.to_owned();
                user.contents.password_hash = password_hash;
                user.update_async(&users).await.unwrap();
                "User exists, password has been updated".into()
            }
        }
    }
}

pub async fn health_check() -> StatusCode {
    StatusCode::OK
}
