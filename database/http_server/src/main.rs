use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Extension, Json, Router,
};
use bonsaidb::{
    core::{
        connection::{AsyncConnection, AsyncStorageConnection, SensitiveString},
        schema::SerializedCollection,
    },
    server::CustomServer,
};
use secrecy::ExposeSecret;
use std::{path::PathBuf, sync::Arc, time::Duration};
use tokio::{sync::RwLock, task::JoinHandle};
use tower_http::add_extension::AddExtensionLayer;
mod database;
use database_common::schema;

struct HostedDatabase {
    inner: CurrentDatabase,
    number: u32,
}

enum CurrentDatabase {
    Setup {
        server: CustomServer,
        handle: JoinHandle<Result<(), bonsaidb::server::Error>>,
    },
    None,
}

impl CurrentDatabase {
    // TODO fix this returning false on subsequent restarted servers after the first
    fn running(&self) -> bool {
        match self {
            Self::Setup { handle, .. } => !handle.is_finished(),
            Self::None => false,
        }
    }

    fn server(&self) -> Option<CustomServer> {
        match self {
            Self::Setup { server, .. } => Some(server.clone()),
            Self::None => None,
        }
    }
}

type BackupLocation = Option<String>;

impl HostedDatabase {
    fn running(&self) -> bool {
        self.inner.running()
    }

    fn server(&self) -> Option<CustomServer> {
        self.inner.server()
    }

    // BUG when bare database is restarted with backup - it hangs
    //  if it's restarted twice - it works. reset and then restored - works.
    async fn restart(&mut self, backup: BackupLocation) {
        self.stop().await.unwrap();

        let server = database::server(backup).await.unwrap();

        let handle = {
            let server = server.clone();
            let number = self.number;
            tokio::spawn(async move {
                println!("database server {} is listening on 5645", number);

                // let _ping_handle = tokio::spawn(async move {
                //     loop {
                //         tokio::time::sleep(Duration::from_secs(3)).await;
                //         println!("database server {} ping", number);
                //     }
                // });

                server.listen_on(5645).await
            })
        };

        // println!(
        //     "new handle is_running: {}, id: {:?}",
        //     !handle.is_finished(),
        //     handle
        // );

        self.inner = CurrentDatabase::Setup { server, handle };
        self.number += 1;
    }

    async fn stop(&mut self) -> anyhow::Result<()> {
        if let CurrentDatabase::Setup { handle, server } = &self.inner {
            handle.abort();
            server.shutdown(Some(Duration::from_secs(1))).await?;
            self.inner = CurrentDatabase::None;
            println!("database has been stopped");
        }
        Ok(())
    }

    async fn reset(&mut self) -> anyhow::Result<()> {
        self.stop().await?;
        if std::path::Path::new(&database_common::storage_location()).exists() {
            std::fs::remove_dir_all(&database_common::storage_location())?;
        }
        println!("database has been reset");
        Ok(())
    }
}

type SharedHostedDatabase = Arc<RwLock<HostedDatabase>>;

#[tokio::main]
async fn main() -> hyper::Result<()> {
    #[allow(unused_mut)]
    let mut hosted_database = HostedDatabase {
        inner: CurrentDatabase::None,
        number: 0,
    };

    // hosted_database.restart(None).await;

    let hosted_database = SharedHostedDatabase::new(RwLock::new(hosted_database));

    let app = Router::new()
        .route("/", get(root))
        .route("/health", get(health_check))
        .route("/cert", get(certificate))
        .route("/database/info", get(database_info))
        .route("/database/users/", post(update_database_user_password))
        .route("/database/backup", get(backup_database))
        .route("/database/restart", post(restart_database))
        .route("/database/stop", get(stop_database))
        .route("/database/reset", get(reset_database))
        .route("/users/", post(replace_dashboard_user))
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
        is_running: database_server.read().await.running(),
    })
}

#[axum_macros::debug_handler]
async fn replace_dashboard_user(
    Extension(database_server): Extension<SharedHostedDatabase>,
    Json(form): Json<interfacing::LoginForm>,
) -> String {
    use argon2::{password_hash::SaltString, Algorithm, Argon2, Params, PasswordHasher, Version};
    let users = database_server
        .read()
        .await
        .server()
        .unwrap()
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
            format!("user {username:?} has been created").into()
        }
        Some(user) => {
            let current_password_hash = user.document.contents.password_hash.as_str();
            // dbg!(&password_hash);
            // dbg!(&current_password_hash);
            if password_hash == current_password_hash {
                // TODO copare hashes correctly
                format!("password of dashboard user {username:?} has NOT been updated").into()
            } else {
                let mut user = user.document.to_owned();
                user.contents.password_hash = password_hash;
                user.update_async(&users).await.unwrap();
                format!("password of dashboard user {username:?} has been updated").into()
            }
        }
    }
}

pub async fn health_check() -> StatusCode {
    StatusCode::OK
}

#[axum_macros::debug_handler]
async fn update_database_user_password(
    Extension(database_server): Extension<SharedHostedDatabase>,
    Json(form): Json<interfacing::LoginForm>,
) -> Response {
    if form.username != "admin" {
        return StatusCode::BAD_REQUEST.into_response();
    }

    let server = database_server.read().await.server().unwrap().clone();
    let user_id = match server.create_user("admin").await {
        Ok(user_id) => user_id,
        Err(bonsaidb::core::Error::UniqueKeyViolation {
            existing_document, ..
        }) => existing_document.id.deserialize().unwrap(),
        Err(_other) => todo!(),
    };

    let _: () = server
        .set_user_password(
            user_id,
            SensitiveString(form.password.expose_secret().to_owned()),
        )
        .await
        .unwrap();

    format!(
        "password of database user {:?} has been updated",
        form.username
    )
    .into_response()
}

#[axum_macros::debug_handler]
async fn backup_database(Extension(database_server): Extension<SharedHostedDatabase>) {
    let server = database_server.read().await.server().unwrap().clone();
    let backup_path = PathBuf::from("backup");
    server.backup(backup_path.clone()).await.unwrap();
    std::fs::copy(
        database_common::storage_location().join(database_common::public_certificate_name()),
        backup_path.join(database_common::public_certificate_name()),
    )
    .unwrap();
}

#[axum_macros::debug_handler]
async fn restart_database(
    Extension(database_server): Extension<SharedHostedDatabase>,
    Json(form): Json<interfacing::DatabaseBackup>,
) {
    database_server.write().await.restart(form.location).await;
}

#[axum_macros::debug_handler]
async fn stop_database(Extension(database_server): Extension<SharedHostedDatabase>) {
    database_server.write().await.stop().await.unwrap();
}

#[axum_macros::debug_handler]
async fn reset_database(Extension(database_server): Extension<SharedHostedDatabase>) {
    database_server.write().await.reset().await.unwrap();
}

#[axum_macros::debug_handler]
async fn certificate() -> Result<Vec<u8>, StatusCode> {
    // let q = "server-data.bonsaidb";
    match std::fs::read("server-data.bonsaidb/pinned-certificate.der") {
        Err(_e) => Err(StatusCode::NOT_FOUND),
        Ok(data) => Ok(data),
    }
}
