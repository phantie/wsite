use backend::conf::{self};
use backend::serve_files;
use backend::startup::Application;
use backend::trace;

#[tokio::main]
async fn main() -> hyper::Result<()> {
    let env = conf::Env::derive();
    let env_conf = conf::EnvConf::derive(env);

    trace::TracingSubscriber::new()
        .pretty(env_conf.log.pretty)
        .set_global_default();

    tracing::debug!("Env: {}", env);
    tracing::debug!("{:?}", env_conf);

    let conf = conf::Conf::new(env, env_conf);

    let application = Application::build(conf).await;

    for f in serve_files::FRONTEND_DIR.files() {
        let path = f.path().to_str().expect("paths to be normal");
        let size = human_bytes::human_bytes(f.contents().len() as f64);
        let served_file = serve_files::ServedFile { path, size: &size };
        tracing::info!("Serving frontend file: {:?}", served_file);
    }

    application.server().await
}
