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

    let dir = conf.dir.clone();
    let dir = std::path::Path::new(&dir);

    let application = Application::build(conf).await;

    // TODO impr
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries {
            if let Ok(dir_entry) = entry {
                if let Ok(metadata) = dir_entry.metadata() {
                    let size = human_bytes::human_bytes(metadata.len() as f64);
                    let path = dir_entry.path();
                    let served_file = serve_files::ServedFile {
                        path: path.to_str().unwrap(),
                        size: &size,
                    };
                    tracing::info!("Serving frontend file/dir: {:?}", served_file);
                }
            }
        }
    }

    application.server().await
}
