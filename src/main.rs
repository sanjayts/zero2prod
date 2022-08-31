use secrecy::ExposeSecret;
use sqlx::PgPool;
use std::env;
use std::net::TcpListener;
use zero2prod::configuration::get_configuration;
use zero2prod::startup::run;
use zero2prod::telemetry::{get_jaegar_subscriber, get_tracing_subscriber, init_tracing_logging};

const APP_NAME: &str = "zero2prod";

#[tokio::main]
async fn main() -> std::io::Result<()> {
    setup_tracing();

    let settings = get_configuration().expect("Failed to read configuration");
    let address = format!(
        "{}:{}",
        settings.application.host, settings.application.port
    );
    let listener = TcpListener::bind(address)?;
    let connection = PgPool::connect_lazy(settings.database.connection_string().expose_secret())
        .expect("Failed to grab connection");
    let server = run(listener, connection)?;
    server.await
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
