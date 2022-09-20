use crate::configuration::{DatabaseSettings, Settings};
use crate::email_client::EmailClient;
use actix_web::dev::Server;
use actix_web::web::Data;
use actix_web::{web, App, HttpServer};
use sqlx::postgres::PgPoolOptions;
use sqlx::{PgPool, Pool, Postgres};
use std::net::TcpListener;
use std::time::Duration;
use tracing_actix_web::TracingLogger;

use crate::routes::*;

pub struct Application {
    port: u16,
    pub server: Server,
}

pub struct ApplicationBaseUrl(pub String);

pub fn run(
    listener: TcpListener,
    conn_pool: PgPool,
    email_client: EmailClient,
    base_url: String,
) -> std::io::Result<Server> {
    let conn_pool = Data::new(conn_pool);
    let email_client = Data::new(email_client);
    let base_url = Data::new(ApplicationBaseUrl(base_url));
    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .route("/subscriptions", web::post().to(subscribe))
            .route("/health_check", web::get().to(health))
            .route("/subscriptions/confirm", web::get().to(confirm))
            .app_data(conn_pool.clone())
            .app_data(email_client.clone())
            .app_data(base_url.clone())
    })
    .listen(listener)?
    .run();
    Ok(server)
}

impl Application {
    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn build(configuration: Settings) -> std::io::Result<Application> {
        let address = format!(
            "{}:{}",
            configuration.application.host, configuration.application.port
        );

        let listener = TcpListener::bind(address)?;
        let port = listener.local_addr().unwrap().port();

        let db_pool = get_connection_pool(configuration.database).await;

        let sender = configuration
            .email_client
            .sender_email()
            .expect("Failed to load sender email");

        let timeout = configuration.email_client.timeout();
        let email_client = EmailClient::new(
            configuration.email_client.base_url,
            sender,
            configuration.email_client.auth_token,
            timeout,
        );
        let server = run(
            listener,
            db_pool,
            email_client,
            configuration.application.base_url,
        )?;
        Ok(Self { port, server })
    }

    pub async fn run_until_stopped(self) -> std::io::Result<()> {
        self.server.await
    }
}

pub async fn get_connection_pool(database_config: DatabaseSettings) -> Pool<Postgres> {
    PgPoolOptions::new()
        .acquire_timeout(Duration::from_secs(2))
        .connect_with(database_config.with_db())
        .await
        .expect("Failed to create pool")
}
