use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::net::TcpListener;
use zero2prod::configuration::{get_configuration, DatabaseSettings};
use zero2prod::startup::run;

/// We perform our integration testing by running against a *real* instance of Postgres spawned
/// by our helper script (scripts/init_db.sh) which is turn is managed by podman. To ensure
/// all tests start with a clean slate, we have a few ways:
///     1. Delete the table data at the start of every test (delete from subscriptions)
///     2. Wrap our test in a transaction and roll back once done
///     3. Create a new database for each test invocation. We will go with this approach
///         But *DO NOTE* please remember to restart your containers reguarly locally to
///         ensure thousands of databases created after test runs aren't bogging you down!!

const HOST: &str = "127.0.0.1";

struct TestData {
    address: String,
    conn_pool: PgPool,
}

#[tokio::test]
async fn health_check_works() {
    let test_data = spawn_app().await;

    let client = reqwest::Client::new();
    let resp = client
        .get(format!("{}/health_check", test_data.address))
        .send()
        .await
        .expect("Failed to send request");
    assert!(resp.status().is_success());
    assert_eq!(resp.content_length(), Some(0));
}

#[tokio::test]
async fn subscribe_returns_200_for_valid_data() {
    let test_data = spawn_app().await;

    let body = "name=Sanjay%20Sharma&email=sanjay_sharma%40hotmail.com";
    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{}/subscriptions", test_data.address))
        .body(body)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(200, resp.status().as_u16());

    let record = sqlx::query!("SELECT name, email FROM subscriptions")
        .fetch_one(&test_data.conn_pool)
        .await
        .expect("Failed to fetch subscriber");

    assert_eq!(record.name, "Sanjay Sharma");
    assert_eq!(record.email, "sanjay_sharma@hotmail.com");
}

#[tokio::test]
async fn subscribe_returns_400_for_invalid_data() {
    let test_data = spawn_app().await;
    let test_cases = vec![
        ("", "missing both name and email"),
        ("name=Sanjay%20Sharma", "missing email"),
        ("email=sanjay_sharma%40hotmail.com", "missing name"),
    ];

    for (invalid_data, error_message) in test_cases {
        let client = reqwest::Client::new();
        let resp = client
            .post(format!("{}/subscriptions", test_data.address))
            .body(invalid_data)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .send()
            .await
            .expect("Failed to send request");

        assert_eq!(
            400,
            resp.status().as_u16(),
            "Expecting failure but got status `{}` for input `{}`",
            resp.status(),
            error_message
        );
    }
}

async fn spawn_app() -> TestData {
    let listener = TcpListener::bind(format!("{}:0", HOST)).unwrap();
    let mut settings = get_configuration().expect("Failed to load config");

    // Randomize the database name
    settings.database.database_name = uuid::Uuid::new_v4().to_string();

    let pool = configure_pool_with_db(&settings.database).await;

    let port = listener.local_addr().unwrap().port();
    let server = run(listener, pool.clone()).expect("Failed to retrieve server");
    let _ = tokio::spawn(server);
    TestData {
        address: format!("http://{}:{}", HOST, port),
        conn_pool: pool,
    }
}

async fn configure_pool_with_db(settings: &DatabaseSettings) -> PgPool {
    let mut connection = PgConnection::connect(&settings.connection_string_without_db())
        .await
        .expect("Failed to create connection to Postgres");
    let db_create_query = format!(r#"CREATE DATABASE "{}""#, &settings.database_name);
    connection
        .execute(db_create_query.as_str())
        .await
        .expect("Failed to create test database");

    let conn_pool = PgPool::connect(settings.connection_string().as_str())
        .await
        .expect("Failed to create connection pool");
    sqlx::migrate!("./migrations")
        .run(&conn_pool)
        .await
        .expect("Failed to migrate");
    conn_pool
}
