use browserflare::{extract_json, test_config, JsonPayload, ResponseFormat};
use reqwest::Client;
use serde_json::json;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn payload_with_prompt() -> JsonPayload {
    JsonPayload {
        url: Some("https://example.com".into()),
        prompt: Some("Get me the list of AI products".into()),
        ..Default::default()
    }
}

fn payload_with_schema() -> JsonPayload {
    JsonPayload {
        url: Some("https://example.com".into()),
        prompt: Some("Get me the list of AI products".into()),
        response_format: Some(ResponseFormat {
            format_type: "json_schema".into(),
            schema: json!({
                "type": "object",
                "properties": {
                    "products": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "name": { "type": "string" },
                                "link": { "type": "string" }
                            },
                            "required": ["name"]
                        }
                    }
                }
            }),
        }),
        ..Default::default()
    }
}

#[tokio::test]
async fn extract_json_with_prompt_only() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({
                "success": true,
                "result": {
                    "AI Products": [
                        "Workers AI",
                        "Vectorize",
                        "AI Gateway"
                    ]
                }
            })),
        )
        .mount(&server)
        .await;

    let config = test_config(&server.uri());
    let client = Client::new();
    let result = extract_json(&client, &config, &payload_with_prompt())
        .await
        .unwrap();
    assert!(result.success);

    let data = result.result.unwrap();
    let products = data["AI Products"].as_array().unwrap();
    assert_eq!(products.len(), 3);
    assert_eq!(products[0], "Workers AI");
}

#[tokio::test]
async fn extract_json_with_schema() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({
                "success": true,
                "result": {
                    "products": [
                        { "name": "Workers AI", "link": "https://developers.cloudflare.com/workers-ai/" },
                        { "name": "Vectorize", "link": "https://developers.cloudflare.com/vectorize/" }
                    ]
                }
            })),
        )
        .mount(&server)
        .await;

    let config = test_config(&server.uri());
    let client = Client::new();
    let result = extract_json(&client, &config, &payload_with_schema())
        .await
        .unwrap();
    assert!(result.success);

    let data = result.result.unwrap();
    let products = data["products"].as_array().unwrap();
    assert_eq!(products.len(), 2);
    assert_eq!(products[0]["name"], "Workers AI");
    assert_eq!(
        products[1]["link"],
        "https://developers.cloudflare.com/vectorize/"
    );
}

#[tokio::test]
async fn extract_json_api_error_response() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({
                "success": false,
                "result": null,
                "errors": [{"message": "prompt or response_format is required"}]
            })),
        )
        .mount(&server)
        .await;

    let config = test_config(&server.uri());
    let client = Client::new();
    let result = extract_json(&client, &config, &payload_with_prompt()).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("API error"));
}

#[tokio::test]
async fn extract_json_http_error() {
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
    let result = extract_json(&client, &config, &payload_with_prompt()).await;
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("500"));
}

#[tokio::test]
async fn extract_json_null_result() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({
                "success": true,
                "result": null
            })),
        )
        .mount(&server)
        .await;

    let config = test_config(&server.uri());
    let client = Client::new();
    let result = extract_json(&client, &config, &payload_with_prompt())
        .await
        .unwrap();
    assert!(result.success);
    assert!(result.result.is_none());
}
