use browserflare::{fetch_content, test_config, ContentPayload};
use reqwest::Client;
use serde_json::json;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn payload() -> ContentPayload {
    ContentPayload {
        url: Some("https://example.com".into()),
        ..Default::default()
    }
}

#[tokio::test]
async fn content_returns_html() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({
                "success": true,
                "result": "<html><body>Hello</body></html>",
                "errors": [],
                "meta": { "status": 200, "title": "Example" }
            })),
        )
        .mount(&server)
        .await;

    let config = test_config(&server.uri());
    let client = Client::new();
    let result = fetch_content(&client, &config, &payload()).await.unwrap();
    assert!(result.success);
    assert_eq!(
        result.result.unwrap(),
        "<html><body>Hello</body></html>"
    );
    let meta = result.meta.unwrap();
    assert_eq!(meta.status, Some(200));
    assert_eq!(meta.title.as_deref(), Some("Example"));
}

#[tokio::test]
async fn content_json_error_response() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({
                "success": false,
                "result": null,
                "errors": [{"message": "url is required"}]
            })),
        )
        .mount(&server)
        .await;

    let config = test_config(&server.uri());
    let client = Client::new();
    let result = fetch_content(&client, &config, &payload()).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("API error"));
}

#[tokio::test]
async fn content_http_error() {
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
    let result = fetch_content(&client, &config, &payload()).await;
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("500"));
}

#[tokio::test]
async fn content_with_html_input() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({
                "success": true,
                "result": "<html><body>Rendered</body></html>",
                "errors": [],
                "meta": { "status": 200, "title": "Test" }
            })),
        )
        .mount(&server)
        .await;

    let html_payload = ContentPayload {
        html: Some("<div>Test</div>".into()),
        ..Default::default()
    };

    let config = test_config(&server.uri());
    let client = Client::new();
    let result = fetch_content(&client, &config, &html_payload).await.unwrap();
    assert!(result.success);
    assert_eq!(
        result.result.unwrap(),
        "<html><body>Rendered</body></html>"
    );
}
