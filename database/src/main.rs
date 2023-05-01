use bonsaidb::local::config::Builder;
use bonsaidb::server::{DefaultPermissions, Server, ServerConfiguration};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let server = Server::open(
        ServerConfiguration::new("server-data.bonsaidb")
            .default_permissions(DefaultPermissions::AllowAll)
            .with_schema::<database::shema::Shape>()
            .unwrap(),
    )
    .await?;

    if server.certificate_chain().await.is_err() {
        server.install_self_signed_certificate(true).await?;
    }

    let task_server = server.clone();

    println!("server is listening...");
    task_server.listen_on(5645).await?;

    Ok(())
}
