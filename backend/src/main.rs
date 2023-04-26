use api_aga_in::configuration::get_configuration;
use api_aga_in::startup::Application;
use api_aga_in::telemetry;

#[tokio::main]
async fn main() -> hyper::Result<()> {
    let subscriber = telemetry::TracingSubscriber::new("site").build(std::io::stdout);
    telemetry::init_global_default(subscriber);

    let configuration = get_configuration();
    let application = Application::build(&configuration).await;

    application.server().await
}
