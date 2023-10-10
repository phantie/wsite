use backend::configuration::{self, env_conf, get_env};
use backend::serve_files;
use backend::startup::Application;
use backend::telemetry;

#[tokio::main]
async fn main() -> hyper::Result<()> {
    let subscriber = telemetry::TracingSubscriber::new("site").build(std::io::stdout);
    telemetry::init_global_default(subscriber);

    let env_conf = env_conf();

    tracing::info!("APP_ENVIRONMENT={}", get_env().as_str());

    let conf = configuration::Conf { env: env_conf };

    let application = Application::build(&conf).await;

    for f in serve_files::FRONTEND_DIR.files() {
        let path = f.path().to_str().expect("paths to be normal");
        let size = human_bytes::human_bytes(f.contents().len() as f64);
        let served_file = serve_files::ServedFile { path, size: &size };
        tracing::info!("Serving frontend file: {:?}", served_file);
    }

    application.server().await
}
