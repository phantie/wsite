use bonsaidb::local::config::Builder;
use bonsaidb::server::{Server, ServerConfiguration};
use bonsaidb::{
    core::{
        admin::{PermissionGroup, Role},
        connection::{AsyncStorageConnection, SensitiveString},
        permissions::{
            bonsai::{AuthenticationMethod, BonsaiAction, ServerAction},
            Permissions, Statement,
        },
        schema::{InsertError, SerializedCollection},
    },
    server::CustomServer,
};

use database_common::schema;

pub async fn server() -> anyhow::Result<CustomServer> {
    let server = Server::open(
        ServerConfiguration::new("server-data.bonsaidb")
            .default_permissions(Permissions::from(
                Statement::for_any()
                    .allowing(&BonsaiAction::Server(ServerAction::Connect))
                    .allowing(&BonsaiAction::Server(ServerAction::Authenticate(
                        AuthenticationMethod::PasswordHash,
                    ))),
            ))
            .with_schema::<schema::Shape>()?,
    )
    .await?;

    if server.certificate_chain().await.is_err() {
        server.install_self_signed_certificate(true).await?;
    }

    let admin_username = "admin";
    let admin_password = "1";

    let user_id = match server.create_user(admin_username).await {
        Ok(user_id) => {
            let _: () = server
                .set_user_password(user_id, SensitiveString(admin_password.into()))
                .await?;
            user_id
        }
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

    Ok(server)
}
