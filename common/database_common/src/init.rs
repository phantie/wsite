use std::path::PathBuf;
use std::time::Duration;

use bonsaidb::local::config::Builder;
use bonsaidb::server::{Server, ServerConfiguration};
use bonsaidb::{
    core::{
        admin::{PermissionGroup, Role},
        connection::{AsyncStorageConnection, AuthenticationMethod},
        permissions::{
            bonsai::{BonsaiAction, ServerAction},
            Permissions, Statement,
        },
        schema::{InsertError, SerializedCollection},
    },
    server::CustomServer,
};

use crate::schema;

pub async fn setup_permissions(server: &impl AsyncStorageConnection) -> anyhow::Result<()> {
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

pub async fn setup_certificate(server: &CustomServer) -> anyhow::Result<()> {
    if server.certificate_chain().await.is_err() {
        server.install_self_signed_certificate(true).await?;
    }
    Ok(())
}

pub async fn setup_contents(server: &impl AsyncStorageConnection) -> anyhow::Result<()> {
    // SCHEMA TWEAK
    let _ = server
        .create_database::<schema::Shape>("shapes", true)
        .await?;
    let _ = server
        .create_database::<schema::User>("users", true)
        .await?;
    let _ = server
        .create_database::<schema::Article>("articles", true)
        .await?;
    let _ = server.create_database::<()>("sessions", true).await?;
    Ok(())
}

pub fn register_schemas<C: bonsaidb::local::config::Builder>(conf: C) -> anyhow::Result<C> {
    // SCHEMA TWEAK
    Ok(conf
        .with_schema::<schema::Shape>()?
        .with_schema::<schema::User>()?
        .with_schema::<schema::Article>()?
        .with_schema::<()>()?)
}

pub async fn create_server(
    storage_location: PathBuf,
    backup_location: Option<String>,
) -> anyhow::Result<CustomServer> {
    match &backup_location {
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

    let configuration =
        ServerConfiguration::new(storage_location).default_permissions(Permissions::from(
            Statement::for_any()
                .allowing(&BonsaiAction::Server(ServerAction::Connect))
                .allowing(&BonsaiAction::Server(ServerAction::Authenticate(
                    AuthenticationMethod::PasswordHash,
                ))),
        ));
    let configuration = register_schemas(configuration)?;

    let server = Server::open(configuration).await?;

    setup_contents(&server).await?;

    setup_certificate(&server).await?;

    setup_permissions(&server).await?;

    Ok(server)
}
