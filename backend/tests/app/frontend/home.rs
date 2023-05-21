use crate::helpers::spawn_app;

#[tokio::test]
async fn test_home() {
    let app = spawn_app().await;

    let response = app.get_home().await;

    let text = response.text().await.unwrap();

    assert!(text.contains("<title>Frontend</title>"));
}
