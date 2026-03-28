use browserflare::{fetch_markdown, test_config, MarkdownPayload};
use reqwest::Client;
use serde_json::json;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn payload() -> MarkdownPayload {
    MarkdownPayload {
        url: Some("https://example.com".into()),
        ..Default::default()
    }
}

#[tokio::test]
async fn markdown_returns_text() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({
                "success": true,
                "result": "# Hello World\n\nSome content here.",
                "errors": [],
                "meta": { "status": 200, "title": "Example" }
            })),
        )
        .mount(&server)
        .await;

    let config = test_config(&server.uri());
    let client = Client::new();
    let result = fetch_markdown(&client, &config, &payload()).await.unwrap();
    assert!(result.success);
    assert_eq!(
        result.result.unwrap(),
        "# Hello World\n\nSome content here."
    );
    let meta = result.meta.unwrap();
    assert_eq!(meta.status, Some(200));
    assert_eq!(meta.title.as_deref(), Some("Example"));
}

#[tokio::test]
async fn markdown_json_error_response() {
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
    let result = fetch_markdown(&client, &config, &payload()).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("API error"));
}

#[tokio::test]
async fn markdown_http_error() {
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
    let result = fetch_markdown(&client, &config, &payload()).await;
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("500"));
}

#[tokio::test]
async fn markdown_with_html_input() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({
                "success": true,
                "result": "# Test\n\nRendered content",
                "errors": [],
                "meta": { "status": 200, "title": "Test" }
            })),
        )
        .mount(&server)
        .await;

    let html_payload = MarkdownPayload {
        html: Some("<div>Test</div>".into()),
        ..Default::default()
    };

    let config = test_config(&server.uri());
    let client = Client::new();
    let result = fetch_markdown(&client, &config, &html_payload).await.unwrap();
    assert!(result.success);
    assert_eq!(
        result.result.unwrap(),
        "# Test\n\nRendered content"
    );
}
