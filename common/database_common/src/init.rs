use std::path::PathBuf;
use std::time::Duration;

use bonsaidb::core::connection::AuthenticationMethod;
use bonsaidb::local::config::Builder;
use bonsaidb::server::{DefaultPermissions, Server, ServerConfiguration, ServerDatabase};
use bonsaidb::{
    core::{
        admin::{PermissionGroup, Role},
        connection::AsyncStorageConnection,
        permissions::{
            bonsai::{BonsaiAction, ServerAction},
            Permissions, Statement,
        },
        schema::{InsertError, SerializedCollection},
    },
    server::CustomServer,
};

use crate::schema;

async fn setup_permissions(server: &CustomServer) -> anyhow::Result<()> {
    let admin_username = "admin";

    let user_id = match server.create_user(admin_username).await {
        Ok(user_id) => user_id,
        Err(bonsaidb::core::Error::UniqueKeyViolation {
            existing_document, ..
        }) => existing_document.id.deserialize()?,
        Err(other) => anyhow::bail!(other),
    };

    let admin_database = server.admin().await;

    let superusers_group_id = match (PermissionGroup {
        name: String::from("superusers"),
        statements: vec![Statement::allow_all_for_any_resource()],
    }
    .push_into_async(&admin_database)
    .await)
    {
        Ok(doc) => doc.header.id,
        Err(InsertError {
            error:
                bonsaidb::core::Error::UniqueKeyViolation {
                    existing_document, ..
                },
            ..
        }) => existing_document.id.deserialize()?,
        Err(other) => anyhow::bail!(other),
    };

    let _superuser_role_id = match (Role {
        name: String::from("superuser"),
        groups: vec![superusers_group_id],
    }
    .push_into_async(&admin_database)
    .await)
    {
        Ok(doc) => doc.header.id,
        Err(InsertError {
            error:
                bonsaidb::core::Error::UniqueKeyViolation {
                    existing_document, ..
                },
            ..
        }) => existing_document.id.deserialize()?,
        Err(other) => anyhow::bail!(other),
    };

    let administrators_group_id = match (PermissionGroup {
        name: String::from("administrators"),
        statements: vec![
            Statement::for_any().allowing(&BonsaiAction::Server(ServerAction::AssumeIdentity))
        ],
    }
    .push_into_async(&admin_database)
    .await)
    {
        Ok(doc) => doc.header.id,
        Err(InsertError {
            error:
                bonsaidb::core::Error::UniqueKeyViolation {
                    existing_document, ..
                },
            ..
        }) => existing_document.id.deserialize()?,
        Err(other) => anyhow::bail!(other),
    };

    // Make our user a member of the administrators group.
    server
        .add_permission_group_to_user(user_id, administrators_group_id)
        .await?;

    println!("admin username: {}", admin_username);

    Ok(())
}

async fn setup_certificate(server: &CustomServer) -> anyhow::Result<()> {
    if server.certificate_chain().await.is_err() {
        server.install_self_signed_certificate(true).await?;
    }
    Ok(())
}

async fn setup_contents(server: &CustomServer) -> anyhow::Result<()> {
    // SCHEMA TWEAK
    let _: ServerDatabase = server
        .create_database::<schema::Shape>("shapes", true)
        .await?;
    let _: ServerDatabase = server
        .create_database::<schema::User>("users", true)
        .await?;
    let _: ServerDatabase = server
        .create_database::<schema::Article>("articles", true)
        .await?;
    let _: ServerDatabase = server.create_database::<()>("sessions", true).await?;
    Ok(())
}

fn register_schemas(conf: ServerConfiguration) -> anyhow::Result<ServerConfiguration> {
    // SCHEMA TWEAK
    Ok(conf
        .with_schema::<schema::Shape>()?
        .with_schema::<schema::User>()?
        .with_schema::<schema::Article>()?
        .with_schema::<()>()?)
}

pub async fn test_server() -> anyhow::Result<CustomServer> {
    let storage_location = tempdir::TempDir::new("test_db_server").unwrap().into_path();
    let configuration = ServerConfiguration::new(storage_location)
        .default_permissions(DefaultPermissions::AllowAll);
    let configuration = register_schemas(configuration)?;

    let server = Server::open(configuration).await?;
    setup_contents(&server).await?;
    setup_certificate(&server).await?;
    Ok(server)
}

pub async fn server(
    storage_location: PathBuf,
    backup: Option<String>,
) -> anyhow::Result<CustomServer> {
    match &backup {
        None => {}
        Some(backup_location) => {
            let tmp_dir = tempdir::TempDir::new("restored-server-data")?;
            let configuration = ServerConfiguration::new(tmp_dir.path());
            let configuration = register_schemas(configuration)?;

            let server = Server::open(configuration).await?;

            let backup_location = PathBuf::from(backup_location);
            server.restore(backup_location.clone()).await?;

            std::fs::copy(
                backup_location.join(crate::public_certificate_name()),
                tmp_dir.path().join(crate::public_certificate_name()),
            )?;

            server.shutdown(Some(Duration::from_secs(1))).await?;

            if std::path::Path::new(&storage_location).exists() {
                std::fs::remove_dir_all(&storage_location)?;
            }

            std::fs::rename(tmp_dir, &storage_location)?;
        }
    };

    let permissions = Permissions::from(
        Statement::for_any()
            .allowing(&BonsaiAction::Server(ServerAction::Connect))
            .allowing(&BonsaiAction::Server(ServerAction::Authenticate(
                AuthenticationMethod::PasswordHash,
            ))),
    );

    let configuration = ServerConfiguration::new(storage_location).default_permissions(permissions);
    let configuration = register_schemas(configuration)?;

    let server = Server::open(configuration).await?;

    setup_contents(&server).await?;

    setup_certificate(&server).await?;

    setup_permissions(&server).await?;

    Ok(server)
}
