use browserflare::{scrape, test_config, ScrapeElement, ScrapePayload};
use reqwest::Client;
use serde_json::json;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn payload() -> ScrapePayload {
    ScrapePayload {
        url: Some("https://example.com".into()),
        elements: vec![
            ScrapeElement { selector: "h1".into() },
            ScrapeElement { selector: "a".into() },
        ],
        ..Default::default()
    }
}

#[tokio::test]
async fn scrape_returns_elements() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({
                "success": true,
                "result": [
                    {
                        "selector": "h1",
                        "results": [
                            {
                                "attributes": [],
                                "height": 39,
                                "html": "Example Domain",
                                "left": 100,
                                "text": "Example Domain",
                                "top": 133.4375,
                                "width": 600
                            }
                        ]
                    },
                    {
                        "selector": "a",
                        "results": [
                            {
                                "attributes": [
                                    { "name": "href", "value": "https://www.iana.org/domains/example" }
                                ],
                                "height": 20,
                                "html": "More information...",
                                "left": 100,
                                "text": "More information...",
                                "top": 249.875,
                                "width": 142
                            }
                        ]
                    }
                ]
            })),
        )
        .mount(&server)
        .await;

    let config = test_config(&server.uri());
    let client = Client::new();
    let result = scrape(&client, &config, &payload()).await.unwrap();
    assert!(result.success);

    let selectors = result.result.unwrap();
    assert_eq!(selectors.len(), 2);

    assert_eq!(selectors[0].selector, "h1");
    assert_eq!(selectors[0].results.len(), 1);
    assert_eq!(selectors[0].results[0].text.as_deref(), Some("Example Domain"));
    assert_eq!(selectors[0].results[0].html.as_deref(), Some("Example Domain"));
    assert_eq!(selectors[0].results[0].height, Some(39.0));
    assert_eq!(selectors[0].results[0].width, Some(600.0));

    assert_eq!(selectors[1].selector, "a");
    assert_eq!(selectors[1].results[0].attributes.len(), 1);
    assert_eq!(selectors[1].results[0].attributes[0].name, "href");
    assert_eq!(
        selectors[1].results[0].attributes[0].value,
        "https://www.iana.org/domains/example"
    );
}

#[tokio::test]
async fn scrape_json_error_response() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({
                "success": false,
                "result": null,
                "errors": [{"message": "elements is required"}]
            })),
        )
        .mount(&server)
        .await;

    let config = test_config(&server.uri());
    let client = Client::new();
    let result = scrape(&client, &config, &payload()).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("API error"));
}

#[tokio::test]
async fn scrape_http_error() {
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
    let result = scrape(&client, &config, &payload()).await;
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("500"));
}

#[tokio::test]
async fn scrape_empty_results() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({
                "success": true,
                "result": [
                    {
                        "selector": "h1",
                        "results": []
                    }
                ]
            })),
        )
        .mount(&server)
        .await;

    let config = test_config(&server.uri());
    let client = Client::new();
    let result = scrape(&client, &config, &payload()).await.unwrap();
    assert!(result.success);

    let selectors = result.result.unwrap();
    assert_eq!(selectors.len(), 1);
    assert_eq!(selectors[0].selector, "h1");
    assert!(selectors[0].results.is_empty());
}
