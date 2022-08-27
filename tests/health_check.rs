use actix_web::web::to;
use sqlx::{query, Connection, PgConnection};
use std::net::TcpListener;
use zero2prod::configuration::get_configuration;
use zero2prod::startup::run;

const HOST: &str = "127.0.0.1";

#[tokio::test]
async fn health_check_works() {
    let address = spawn_app();

    let client = reqwest::Client::new();
    let resp = client
        .get(format!("{}/health_check", address))
        .send()
        .await
        .expect("Failed to send request");
    assert!(resp.status().is_success());
    assert_eq!(resp.content_length(), Some(0));
}

#[tokio::test]
async fn subscribe_returns_200_for_valid_data() {
    let address = spawn_app();

    let body = "name=Sanjay%20Sharma&email=sanjay_sharma%40hotmail.com";
    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{}/subscriptions", address))
        .body(body)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(200, resp.status().as_u16());

    let settings = get_configuration().expect("Failed to load configuration");
    let mut connection = PgConnection::connect(&settings.database.connection_string())
        .await
        .expect("Failed to get connection");

    // let record = sqlx::query!("SELECT name, email FROM subscriptions")
    //     .fetch_one(&mut connection)
    //     .await
    //     .expect("Failed to fetch subscriber");
}

#[tokio::test]
async fn subscribe_returns_400_for_invalid_data() {
    let address = spawn_app();
    let test_cases = vec![
        ("", "missing both name and email"),
        ("name=Sanjay%20Sharma", "missing email"),
        ("email=sanjay_sharma%40hotmail.com", "missing name"),
    ];

    for (invalid_data, error_message) in test_cases {
        let client = reqwest::Client::new();
        let resp = client
            .post(format!("{}/subscriptions", address))
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

fn spawn_app() -> String {
    let listener = TcpListener::bind(format!("{}:0", HOST)).unwrap();
    let port = listener.local_addr().unwrap().port();
    let server = run(listener).expect("Failed to retrieve server");
    let _ = tokio::spawn(server);
    format!("http://{}:{}", HOST, port)
}
