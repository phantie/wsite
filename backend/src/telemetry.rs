use tracing::{subscriber::set_global_default, Subscriber};
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::{
    filter,
    fmt::MakeWriter,
    layer::{Layer, SubscriberExt},
    EnvFilter, Registry,
};

static BACKEND_CRATE_NAME: &str = "backend";

pub struct TracingSubscriber {
    name: String,
    env_filter: String,
}

impl TracingSubscriber {
    pub fn new<T>(name: T) -> Self
    where
        T: AsRef<str>,
    {
        Self {
            name: name.as_ref().to_string(),
            env_filter: "info".into(),
        }
    }

    /// Creates a [`tracing::Subscriber`] configured to format logs with [`Bunyan`]
    ///
    /// [`Bunyan`]: https://docs.rs/tracing-bunyan-formatter/latest/tracing_bunyan_formatter/
    pub fn build<Sink>(self, sink: Sink) -> impl Subscriber + Sync + Send
    where
        Sink: for<'a> MakeWriter<'a> + Sync + Send + 'static,
    {
        let logging_layer = {
            let skip_fields = ["file", "line", "target"];

            let formatting_layer = BunyanFormattingLayer::new(self.name.clone(), sink)
                .skip_fields(skip_fields.into_iter().map(|s| s.to_owned()).into_iter())
                .expect("unable to build the bunyan formatting layer");

            // let formatting_layer = tracing_subscriber::fmt::layer();

            let env_filter = EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new(self.env_filter));

            let target_filter = filter::Targets::new()
                .with_target(BACKEND_CRATE_NAME, tracing::Level::INFO)
                .with_target("tower_http::trace", tracing::Level::INFO)
                .with_target("mio::poll", filter::LevelFilter::OFF);
            formatting_layer
                .with_filter(target_filter)
                .with_filter(env_filter)

            // formatting_layer
        };

        Registry::default()
            .with(JsonStorageLayer)
            .with(logging_layer)
    }
}

/// Sets `subscriber` as the global default [`tracing::Subscriber`].
pub fn init_global_default(subscriber: impl Subscriber + Sync + Send) {
    LogTracer::init().expect("Failed to set logger");
    set_global_default(subscriber).expect("Failed to set subscriber");
}

/// Spawns a blocking task in the scope of the current tracing span.
pub fn spawn_blocking_with_tracing<F, R>(f: F) -> tokio::task::JoinHandle<R>
where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
{
    let current_span = tracing::Span::current();
    tokio::task::spawn_blocking(move || current_span.in_scope(f))
}
