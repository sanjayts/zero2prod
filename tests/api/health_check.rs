use crate::helpers::spawn_app;

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
