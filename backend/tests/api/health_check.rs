use crate::helpers::spawn_app;
use hyper::StatusCode;
use serial_test::serial;

#[serial]
#[tokio::test]
async fn health_check_works() {
    let app = spawn_app().await;

    let client = reqwest::Client::new();
    let response = client
        .get(app.address.with_api_path("/health_check"))
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(StatusCode::OK, response.status());
    assert_eq!(Some(0), response.content_length());
}
