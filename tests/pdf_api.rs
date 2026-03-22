use browserflare::{generate_pdf, test_config, PdfPayload};
use reqwest::Client;
use serde_json::json;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn payload() -> PdfPayload {
    PdfPayload {
        url: Some("https://example.com".into()),
        ..Default::default()
    }
}

#[tokio::test]
async fn pdf_returns_bytes() {
    let server = MockServer::start().await;
    let fake_pdf = b"%PDF-1.4 fake".to_vec();

    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_bytes(fake_pdf.clone())
                .insert_header("content-type", "application/pdf"),
        )
        .mount(&server)
        .await;

    let config = test_config(&server.uri());
    let client = Client::new();
    let result = generate_pdf(&client, &config, &payload()).await.unwrap();
    assert_eq!(result.bytes, fake_pdf);
    assert_eq!(result.content_type, "application/pdf");
}

#[tokio::test]
async fn pdf_json_error_response() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({
                    "errors": [{"message": "missing url or html"}]
                }))
                .insert_header("content-type", "application/json"),
        )
        .mount(&server)
        .await;

    let config = test_config(&server.uri());
    let client = Client::new();
    let result = generate_pdf(&client, &config, &payload()).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("API error"));
}

#[tokio::test]
async fn pdf_http_500_error() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(
            ResponseTemplate::new(500)
                .set_body_string("server overloaded")
                .insert_header("content-type", "text/plain"),
        )
        .mount(&server)
        .await;

    let config = test_config(&server.uri());
    let client = Client::new();
    let result = generate_pdf(&client, &config, &payload()).await;
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("500"));
}

#[tokio::test]
async fn pdf_from_html_payload() {
    let server = MockServer::start().await;
    let fake_pdf = b"%PDF".to_vec();

    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_bytes(fake_pdf.clone())
                .insert_header("content-type", "application/pdf"),
        )
        .mount(&server)
        .await;

    let html_payload = PdfPayload {
        html: Some("<h1>Hello</h1>".into()),
        ..Default::default()
    };

    let config = test_config(&server.uri());
    let client = Client::new();
    let result = generate_pdf(&client, &config, &html_payload).await.unwrap();
    assert_eq!(result.bytes, fake_pdf);
}
