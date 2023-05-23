use api_aga_in::configuration::{self, env_conf, get_env};
use api_aga_in::startup::Application;

#[tokio::main]
async fn main() -> hyper::Result<()> {
    let subscriber = telemetry::TracingSubscriber::new("site").build(std::io::stdout);
    telemetry::init_global_default(subscriber);

    let env_conf = env_conf();

    tracing::info!("APP_ENVIRONMENT={}", get_env().as_str());

    let conf = configuration::Conf {
        db_client: configuration::DbClientConf::Normal {
            quic_url: format!("bonsaidb://{}:{}", env_conf.db.host, env_conf.db.port),
            password: env_conf
                .db
                .password
                .clone()
                .expect("db password must be specified"),
            info_server: env_conf.db.clone().into(),
        },
        env: env_conf,
    };

    let application = Application::build(&conf).await;

    application.server().await
}
