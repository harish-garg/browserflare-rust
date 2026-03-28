use browserflare::{take_snapshot, test_config, SnapshotPayload, ScreenshotOptions};
use reqwest::Client;
use serde_json::json;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn url_payload() -> SnapshotPayload {
    SnapshotPayload {
        url: Some("https://example.com".into()),
        ..Default::default()
    }
}

#[tokio::test]
async fn snapshot_returns_content_and_screenshot() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({
                "success": true,
                "result": {
                    "screenshot": "iVBORw0KGgoAAAANSUhEUg==",
                    "content": "<html><body>Hello</body></html>"
                }
            })),
        )
        .mount(&server)
        .await;

    let config = test_config(&server.uri());
    let client = Client::new();
    let result = take_snapshot(&client, &config, &url_payload()).await.unwrap();
    assert!(result.success);
    let data = result.result.unwrap();
    assert_eq!(data.content, "<html><body>Hello</body></html>");
    assert_eq!(data.screenshot, "iVBORw0KGgoAAAANSUhEUg==");
}

#[tokio::test]
async fn snapshot_json_error_response() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({
                "success": false,
                "result": null,
                "errors": [{"message": "url or html is required"}]
            })),
        )
        .mount(&server)
        .await;

    let config = test_config(&server.uri());
    let client = Client::new();
    let result = take_snapshot(&client, &config, &url_payload()).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("API error"));
}

#[tokio::test]
async fn snapshot_http_error() {
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
    let result = take_snapshot(&client, &config, &url_payload()).await;
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("500"));
}

#[tokio::test]
async fn snapshot_with_html_input() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({
                "success": true,
                "result": {
                    "screenshot": "c2NyZWVuc2hvdA==",
                    "content": "<html><body>Rendered</body></html>"
                }
            })),
        )
        .mount(&server)
        .await;

    let html_payload = SnapshotPayload {
        html: Some("<div>Test</div>".into()),
        ..Default::default()
    };

    let config = test_config(&server.uri());
    let client = Client::new();
    let result = take_snapshot(&client, &config, &html_payload).await.unwrap();
    assert!(result.success);
    let data = result.result.unwrap();
    assert_eq!(data.content, "<html><body>Rendered</body></html>");
}

#[tokio::test]
async fn snapshot_with_screenshot_options() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({
                "success": true,
                "result": {
                    "screenshot": "ZnVsbHBhZ2U=",
                    "content": "<html><body>Full</body></html>"
                }
            })),
        )
        .mount(&server)
        .await;

    let payload = SnapshotPayload {
        url: Some("https://example.com".into()),
        screenshot_options: Some(ScreenshotOptions {
            full_page: Some(true),
            ..Default::default()
        }),
        ..Default::default()
    };

    let config = test_config(&server.uri());
    let client = Client::new();
    let result = take_snapshot(&client, &config, &payload).await.unwrap();
    assert!(result.success);
    assert!(result.result.is_some());
}
