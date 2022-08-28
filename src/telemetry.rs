use tracing::Subscriber;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::fmt::MakeWriter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{EnvFilter, Registry};

pub fn init_tracing_logging(subscriber: impl Subscriber + Sync + Send) {
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
    let subscriber = Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer);
    subscriber
}
