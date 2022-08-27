use sqlx::PgPool;
use std::net::TcpListener;
use zero2prod::configuration::get_configuration;
use zero2prod::startup::run;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let settings = get_configuration().expect("Failed to read configuration");
    let address = format!("127.0.0.1:{}", settings.application_port);
    let listener = TcpListener::bind(address)?;
    let connection = PgPool::connect(&settings.database.connection_string())
        .await
        .expect("Failed to grab connection");
    let server = run(listener, connection)?;
    server.await
}
