use crate::helpers::spawn_app;
use hyper::StatusCode;

use common::static_routes::*;

#[tokio::test]
async fn health_check_works() {
    let app = spawn_app().await;

    let response = app
        .get(routes().api.health_check)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(StatusCode::OK, response.status());
    assert_eq!(Some(0), response.content_length());
}
