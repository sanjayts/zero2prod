use crate::helpers::spawn_app;
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};

#[tokio::test]
async fn confirmations_without_token_rejected_with_400() {
    let test_application = spawn_app().await;

    let response = reqwest::get(format!(
        "{}/subscriptions/confirm",
        test_application.address
    ))
    .await
    .unwrap();
    assert_eq!(response.status().as_u16(), 400);
}

#[tokio::test]
async fn link_returned_by_subscribe_returns_200_if_called() {
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

    let response = reqwest::get(confirmation_links.html_link).await.unwrap();
    assert_eq!(response.status().as_u16(), 200);
}

#[tokio::test]
async fn clicking_on_confirmation_link_confirms_subscriber() {
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

    let response = reqwest::get(confirmation_links.html_link)
        .await
        .unwrap()
        .error_for_status()
        .unwrap();

    let saved = sqlx::query!("SELECT email, name, status FROM subscriptions",)
        .fetch_one(&test_application.conn_pool)
        .await
        .expect("Failed to fetch subscription");

    assert_eq!(saved.name, "Sanjay Sharma");
    assert_eq!(saved.email, "sanjay_sharma@hotmail.com");
    assert_eq!(saved.status, "confirmed");
}
