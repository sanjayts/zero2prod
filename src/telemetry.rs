use tracing::Subscriber;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::fmt::MakeWriter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{EnvFilter, Registry};

pub fn init_tracing_logging(subscriber: impl Subscriber + Sync + Send) {
    // required to ensure that all app logs (not just user tracing logs) are spit out in a
    // tracing compatible format
    LogTracer::init().expect("Failed to initialize log tracer");

    tracing::subscriber::set_global_default(subscriber).expect("Failed to set subscriber");
}

pub fn get_tracing_subscriber<S>(
    app_name: &str,
    log_level: &str,
    sink: S,
) -> impl Subscriber + Sync + Send
where
    S: for<'a> MakeWriter<'a> + Sync + Send + 'static,
{
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(log_level));
    let formatting_layer = BunyanFormattingLayer::new(app_name.into(), sink);
    Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer)
}

pub fn get_jaegar_subscriber<S>(
    app_name: &str,
    log_level: &str,
    sink: S,
) -> impl Subscriber + Sync + Send
where
    S: for<'a> MakeWriter<'a> + Sync + Send + 'static,
{
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(log_level));
    let formatting_layer = BunyanFormattingLayer::new(app_name.into(), sink);
    let jaegar_tracer = opentelemetry_jaeger::new_pipeline()
        .with_service_name(app_name)
        .install_simple()
        .expect("Error initializing Jaeger exporter");
    let jaegar_layer = OpenTelemetryLayer::new(jaegar_tracer);
    Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer)
        .with(jaegar_layer)
}
