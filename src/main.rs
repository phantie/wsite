use api_aga_in::configuration::get_configuration;
use api_aga_in::startup::Application;
use api_aga_in::telemetry::{get_subscriber, init_subscriber};

#[tokio::main]
async fn main() -> hyper::Result<()> {
    let subscriber = get_subscriber("api_aga_in".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let configuration = get_configuration();
    let application = Application::build(&configuration).await;

    application.server().await
}
