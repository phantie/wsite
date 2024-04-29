// Tracing definitions
//

use tracing::{subscriber::set_global_default, Subscriber};
use tracing_log::LogTracer;
use tracing_subscriber::{
    filter,
    layer::{Layer, SubscriberExt},
    EnvFilter, Registry,
};

pub struct TracingSubscriber {
    crate_level: tracing::Level,
    rust_log_fallback: String,
    pretty: bool,
}

impl Default for TracingSubscriber {
    fn default() -> Self {
        Self {
            crate_level: tracing::Level::DEBUG,
            rust_log_fallback: "debug".into(),
            pretty: false,
        }
    }
}

impl TracingSubscriber {
    pub fn new() -> Self {
        Self::default()
    }

    #[allow(unused)]
    pub fn crate_level(mut self, value: tracing::Level) -> Self {
        self.crate_level = value;
        self
    }

    #[allow(unused)]
    pub fn rust_log_fallback(mut self, value: impl AsRef<str>) -> Self {
        self.rust_log_fallback = value.as_ref().into();
        self
    }

    #[allow(unused)]
    pub fn pretty(mut self, value: bool) -> Self {
        self.pretty = value;
        self
    }

    pub fn set_global_default(self) {
        LogTracer::init().expect("Failed to set logger");
        set_global_default(self.build()).expect("Failed to set subscriber");
    }

    fn build(self) -> Box<dyn Subscriber + Sync + Send> {
        // depends on RUST_LOG env var
        let env_filter = EnvFilter::try_from_default_env()
            // if unset, use rust_log_fallback
            .or_else(|_| EnvFilter::try_new(self.rust_log_fallback))
            .expect("correct RUST_LOG");

        let target_filter = filter::Targets::new()
            .with_target("server", self.crate_level)
            .with_target("hyper", tracing::Level::INFO)
            .with_default(tracing::Level::TRACE);

        // ugly
        if self.pretty {
            Box::new(
                Registry::default().with(
                    tracing_subscriber::fmt::layer()
                        .pretty()
                        .with_filter(env_filter)
                        .with_filter(target_filter),
                ),
            )
        } else {
            Box::new(
                Registry::default().with(
                    tracing_subscriber::fmt::layer()
                        .with_filter(env_filter)
                        .with_filter(target_filter),
                ),
            )
        }
    }
}

#[derive(Clone, Default)]
pub struct RequestIdProducer {
    counter: std::sync::Arc<std::sync::atomic::AtomicU64>,
}

impl tower_http::request_id::MakeRequestId for RequestIdProducer {
    fn make_request_id<B>(
        &mut self,
        _request: &hyper::http::Request<B>,
    ) -> Option<tower_http::request_id::RequestId> {
        let request_id = self
            .counter
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst)
            .to_string()
            .parse()
            .unwrap();

        Some(tower_http::request_id::RequestId::new(request_id))
    }
}

pub fn request_trace_layer() -> tower::ServiceBuilder<
    tower::layer::util::Stack<
        tower_http::request_id::PropagateRequestIdLayer,
        tower::layer::util::Stack<
            tower_http::trace::TraceLayer<
                tower_http::classify::SharedClassifier<
                    tower_http::classify::ServerErrorsAsFailures,
                >,
                impl Fn(&hyper::Request<hyper::Body>) -> tracing::Span + Clone,
            >,
            tower::layer::util::Stack<
                tower_http::request_id::SetRequestIdLayer<RequestIdProducer>,
                tower::layer::util::Identity,
            >,
        >,
    >,
> {
    pub use tower_http::{
        trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse},
        LatencyUnit, ServiceBuilderExt,
    };

    tower::ServiceBuilder::new()
        .set_x_request_id(RequestIdProducer::default())
        .layer(
            tower_http::trace::TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(tracing::Level::DEBUG).include_headers(true))
                .make_span_with(|request: &hyper::http::Request<hyper::Body>| {
                tracing::debug_span!(
                    "request",
                    method = %request.method(),
                    uri = %request.uri(),
                    version = ?request.version(),
                    request_id = %request.headers().get("x-request-id").unwrap().to_str().unwrap(),
                )
            })
            .on_request(DefaultOnRequest::new().level(tracing::Level::INFO))
            .on_response(
                DefaultOnResponse::new()
                    .level(tracing::Level::INFO)
                    .latency_unit(LatencyUnit::Seconds),
            ),
    )
    .propagate_x_request_id()
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
