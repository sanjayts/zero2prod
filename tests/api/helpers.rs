use once_cell::sync::Lazy;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use wiremock::MockServer;
use zero2prod::configuration::{get_configuration, DatabaseSettings};
use zero2prod::startup::{get_connection_pool, Application};
use zero2prod::telemetry::{get_tracing_subscriber, init_tracing_logging};

/// We perform our integration testing by running against a *real* instance of Postgres spawned
/// by our helper script (scripts/init_db.sh) which is turn is managed by podman. To ensure
/// all tests start with a clean slate, we have a few ways:
///     1. Delete the table data at the start of every test (delete from subscriptions)
///     2. Wrap our test in a transaction and roll back once done
///     3. Create a new database for each test invocation. We will go with this approach
///         But *DO NOTE* please remember to restart your containers regularly locally to
///         ensure thousands of databases created after test runs aren't bogging you down!!

static TRACING: Lazy<()> = Lazy::new(|| {
    let app_name = "integration-test";
    let log_level = "debug";
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_tracing_subscriber(app_name, log_level, std::io::stdout);
        init_tracing_logging(subscriber);
    } else {
        let subscriber = get_tracing_subscriber(app_name, log_level, std::io::sink);
        init_tracing_logging(subscriber);
    }
});

pub struct ConfirmationLinks {
    pub html_link: reqwest::Url,
    pub text_link: reqwest::Url,
}

pub struct TestApplication {
    pub address: String,
    pub conn_pool: PgPool,
    pub email_server: MockServer,
    pub port: u16,
}

impl TestApplication {
    pub async fn post_subscriptions(&self, body: String) -> reqwest::Response {
        let client = reqwest::Client::new();
        client
            .post(format!("{}/subscriptions", &self.address))
            .body(body)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .send()
            .await
            .expect("Failed to send request")
    }

    pub fn get_confirmation_links(&self, email_request: &wiremock::Request) -> ConfirmationLinks {
        let extract_link = |s: &str| {
            let links: Vec<_> = linkify::LinkFinder::new()
                .links(s)
                .filter(|link| *link.kind() == linkify::LinkKind::Url)
                .collect();
            assert_eq!(links.len(), 1);
            let raw_confirmation_link = links[0].as_str().to_owned();
            let mut confirmation_link =
                reqwest::Url::parse(raw_confirmation_link.as_str()).unwrap();

            assert_eq!(confirmation_link.host_str().unwrap(), "127.0.0.1");
            confirmation_link.set_port(Some(self.port)).unwrap();
            confirmation_link
        };
        let body: serde_json::Value = serde_json::from_slice(&email_request.body).unwrap();
        let html_link = extract_link(body["HtmlBody"].as_str().unwrap());
        let text_link = extract_link(body["TextBody"].as_str().unwrap());
        ConfirmationLinks {
            html_link,
            text_link,
        }
    }
}

pub async fn spawn_app() -> TestApplication {
    Lazy::force(&TRACING);

    let email_server = MockServer::start().await;

    let settings = {
        let mut c = get_configuration().expect("Failed to load config");

        // Randomize the database name
        c.database.database_name = uuid::Uuid::new_v4().to_string();

        // use random OS port when spawning the test app
        c.application.port = 0;

        // Use mock server URI has the base URL for our email server
        c.email_client.base_url = email_server.uri();

        c
    };
    let conn_pool = configure_database(settings.database.clone()).await;
    let app = Application::build(settings)
        .await
        .expect("Failed to build app");
    let port = app.port();
    let address = format!("http://127.0.0.1:{}", port);
    let _ = tokio::spawn(app.run_until_stopped());
    TestApplication {
        address,
        conn_pool,
        email_server,
        port,
    }
}

async fn configure_database(settings: DatabaseSettings) -> PgPool {
    let mut connection = PgConnection::connect_with(&settings.without_db())
        .await
        .expect("Failed to create connection to Postgres");
    let db_create_query = format!(r#"CREATE DATABASE "{}""#, &settings.database_name);
    connection
        .execute(db_create_query.as_str())
        .await
        .expect("Failed to create test database");

    let pool = get_connection_pool(settings).await;
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to migrate");
    pool
}
