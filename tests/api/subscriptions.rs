use crate::helpers::spawn_app;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn subscribe_sends_confirmation_mail_with_link() {
    let test_application = spawn_app().await;
    let body = "name=Sanjay%20Sharma&email=sanjay_sharma%40hotmail.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&test_application.email_server)
        .await;

    test_application.post_subscriptions(body.into()).await;

    let mail_request = &test_application
        .email_server
        .received_requests()
        .await
        .unwrap()[0];

    let confirmation_links = test_application.get_confirmation_links(mail_request);
    assert_eq!(confirmation_links.html_link, confirmation_links.text_link);
}

#[tokio::test]
async fn subscribe_returns_200_for_valid_data() {
    let test_application = spawn_app().await;
    let body = "name=Sanjay%20Sharma&email=sanjay_sharma%40hotmail.com";
    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&test_application.email_server)
        .await;

    let response = test_application.post_subscriptions(body.to_string()).await;

    assert_eq!(200, response.status().as_u16());
}

#[tokio::test]
async fn subscribe_persists_the_new_subscriber() {
    let test_application = spawn_app().await;
    let body = "name=Sanjay%20Sharma&email=sanjay_sharma%40hotmail.com";

    let _response = test_application.post_subscriptions(body.to_string()).await;

    let record = sqlx::query!("SELECT name, email, status FROM subscriptions")
        .fetch_one(&test_application.conn_pool)
        .await
        .expect("Failed to fetch subscriber");

    assert_eq!(record.name, "Sanjay Sharma");
    assert_eq!(record.email, "sanjay_sharma@hotmail.com");
    assert_eq!(record.status, "pending_confirmation");
}

#[tokio::test]
async fn subscribe_sends_confirmation_email_valid_data() {
    let test_application = spawn_app().await;
    let body = "name=Sanjay%20Sharma&email=sanjay_sharma%40hotmail.com";
    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&test_application.email_server)
        .await;

    let response = test_application.post_subscriptions(body.to_string()).await;

    assert_eq!(200, response.status().as_u16());
}

#[tokio::test]
async fn subscribe_returns_400_for_invalid_data() {
    let test_application = spawn_app().await;
    let test_cases = vec![
        ("", "missing both name and email"),
        ("name=Sanjay%20Sharma", "missing email"),
        ("email=sanjay_sharma%40hotmail.com", "missing name"),
        ("email=  &name=  ", "empty name and email"),
    ];

    for (body, error_message) in test_cases {
        let response = test_application.post_subscriptions(body.to_string()).await;

        assert_eq!(
            400,
            response.status().as_u16(),
            "Expecting failure but got status `{}` for input `{}`",
            response.status(),
            error_message
        );
    }
}
