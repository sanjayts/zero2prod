use crate::helpers::spawn_app;
use reqwest::Url;
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

    let response = test_application.post_subscriptions(body.into()).await;

    let mail_request = &test_application
        .email_server
        .received_requests()
        .await
        .unwrap()[0];

    let body: serde_json::Value = serde_json::from_slice(&mail_request.body).unwrap();

    let extract_link = |s: &str| {
        let links: Vec<_> = linkify::LinkFinder::new().links(s).collect();
        assert_eq!(links.len(), 1);
        links[0].as_str().to_owned()
    };

    let raw_confirmation_link = extract_link(body["HtmlBody"].as_str().unwrap());
    let confirmation_link = Url::parse(raw_confirmation_link.as_str()).unwrap();

    assert_eq!(confirmation_link.host_str().unwrap(), "127.0.0.1");

    // let response = reqwest::get(confirmation_link).await.unwrap();
    // assert_eq!(response.status().as_u16(), 200);
}
