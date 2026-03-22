use browserflare::{test_config, take_screenshot, ScreenshotPayload};
use reqwest::Client;
use serde_json::json;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn payload() -> ScreenshotPayload {
    ScreenshotPayload {
        url: "https://example.com".into(),
        ..Default::default()
    }
}

#[tokio::test]
async fn screenshot_returns_image_bytes() {
    let server = MockServer::start().await;
    let fake_png = vec![0x89, 0x50, 0x4E, 0x47]; // PNG magic bytes

    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_bytes(fake_png.clone())
                .insert_header("content-type", "image/png"),
        )
        .mount(&server)
        .await;

    let config = test_config(&server.uri());
    let client = Client::new();
    let result = take_screenshot(&client, &config, &payload()).await.unwrap();
    assert_eq!(result.bytes, fake_png);
    assert_eq!(result.content_type, "image/png");
}

#[tokio::test]
async fn screenshot_json_error_response() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({
                    "errors": [{"message": "invalid url"}]
                }))
                .insert_header("content-type", "application/json"),
        )
        .mount(&server)
        .await;

    let config = test_config(&server.uri());
    let client = Client::new();
    let result = take_screenshot(&client, &config, &payload()).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("API error"));
}

#[tokio::test]
async fn screenshot_http_error() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(
            ResponseTemplate::new(500)
                .set_body_string("internal server error")
                .insert_header("content-type", "text/plain"),
        )
        .mount(&server)
        .await;

    let config = test_config(&server.uri());
    let client = Client::new();
    let result = take_screenshot(&client, &config, &payload()).await;
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("500"));
}

#[tokio::test]
async fn screenshot_jpeg_content_type() {
    let server = MockServer::start().await;
    let fake_jpeg = vec![0xFF, 0xD8, 0xFF, 0xE0];

    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_bytes(fake_jpeg.clone())
                .insert_header("content-type", "image/jpeg"),
        )
        .mount(&server)
        .await;

    let config = test_config(&server.uri());
    let client = Client::new();
    let result = take_screenshot(&client, &config, &payload()).await.unwrap();
    assert_eq!(result.content_type, "image/jpeg");
    assert_eq!(result.bytes, fake_jpeg);
}
