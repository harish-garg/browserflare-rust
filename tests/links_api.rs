use browserflare::{fetch_links, test_config, LinksPayload};
use reqwest::Client;
use serde_json::json;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn payload() -> LinksPayload {
    LinksPayload {
        url: Some("https://example.com".into()),
        ..Default::default()
    }
}

#[tokio::test]
async fn links_returns_urls() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({
                "success": true,
                "result": [
                    "https://example.com/page1",
                    "https://example.com/page2",
                    "https://other.com/external"
                ]
            })),
        )
        .mount(&server)
        .await;

    let config = test_config(&server.uri());
    let client = Client::new();
    let result = fetch_links(&client, &config, &payload()).await.unwrap();
    assert!(result.success);
    let links = result.result.unwrap();
    assert_eq!(links.len(), 3);
    assert_eq!(links[0], "https://example.com/page1");
    assert_eq!(links[2], "https://other.com/external");
}

#[tokio::test]
async fn links_json_error_response() {
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
    let result = fetch_links(&client, &config, &payload()).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("API error"));
}

#[tokio::test]
async fn links_http_error() {
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
    let result = fetch_links(&client, &config, &payload()).await;
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("500"));
}

#[tokio::test]
async fn links_with_html_input() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({
                "success": true,
                "result": ["https://example.com/link1"]
            })),
        )
        .mount(&server)
        .await;

    let html_payload = LinksPayload {
        html: Some("<a href='https://example.com/link1'>Link</a>".into()),
        ..Default::default()
    };

    let config = test_config(&server.uri());
    let client = Client::new();
    let result = fetch_links(&client, &config, &html_payload).await.unwrap();
    assert!(result.success);
    assert_eq!(result.result.unwrap().len(), 1);
}

#[tokio::test]
async fn links_visible_only_serializes() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({
                "success": true,
                "result": ["https://example.com/visible"]
            })),
        )
        .mount(&server)
        .await;

    let vis_payload = LinksPayload {
        url: Some("https://example.com".into()),
        visible_links_only: Some(true),
        ..Default::default()
    };

    let config = test_config(&server.uri());
    let client = Client::new();
    let result = fetch_links(&client, &config, &vis_payload).await.unwrap();
    assert!(result.success);
    assert_eq!(result.result.unwrap(), vec!["https://example.com/visible"]);
}

#[tokio::test]
async fn links_empty_result() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({
                "success": true,
                "result": []
            })),
        )
        .mount(&server)
        .await;

    let config = test_config(&server.uri());
    let client = Client::new();
    let result = fetch_links(&client, &config, &payload()).await.unwrap();
    assert!(result.success);
    assert!(result.result.unwrap().is_empty());
}
