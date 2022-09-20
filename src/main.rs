use std::env;
use zero2prod::configuration::get_configuration;
use zero2prod::startup::Application;
use zero2prod::telemetry::{get_jaegar_subscriber, get_tracing_subscriber, init_tracing_logging};

const APP_NAME: &str = "zero2prod";

#[tokio::main]
async fn main() -> std::io::Result<()> {
    setup_tracing();
    let configuration = get_configuration().expect("Failed to read configuration");
    let app = Application::build(configuration)
        .await
        .expect("Failed to build app");
    app.run_until_stopped().await
}

fn setup_tracing() {
    if env::var("JAEGAR_ENABLED").is_ok() {
        let subscriber = get_jaegar_subscriber(APP_NAME, "info", std::io::stdout);
        init_tracing_logging(subscriber);
    } else {
        let subscriber = get_tracing_subscriber(APP_NAME, "info", std::io::stdout);
        init_tracing_logging(subscriber);
    }
}
