use browserflare::{
    cancel_crawl, get_crawl_results_paginated, get_crawl_status, poll_until_complete, start_crawl,
    test_config, CrawlPayload,
};
use reqwest::Client;
use serde_json::json;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn payload() -> CrawlPayload {
    CrawlPayload {
        url: "https://example.com".into(),
        ..Default::default()
    }
}

// ── start_crawl ─────────────────────────────────────────────────────────

#[tokio::test]
async fn start_crawl_returns_job_id() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "success": true,
            "result": "job-abc-123",
            "errors": [],
            "messages": []
        })))
        .mount(&server)
        .await;

    let config = test_config(&server.uri());
    let client = Client::new();
    let job_id = start_crawl(&client, &config, &payload()).await.unwrap();
    assert_eq!(job_id, "job-abc-123");
}

#[tokio::test]
async fn start_crawl_api_error() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(403).set_body_json(json!({
            "success": false,
            "result": null,
            "errors": [{"message": "forbidden"}],
            "messages": []
        })))
        .mount(&server)
        .await;

    let config = test_config(&server.uri());
    let client = Client::new();
    let result = start_crawl(&client, &config, &payload()).await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("API error"));
}

#[tokio::test]
async fn start_crawl_no_result_field() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "success": true,
            "result": null,
            "errors": [],
            "messages": []
        })))
        .mount(&server)
        .await;

    let config = test_config(&server.uri());
    let client = Client::new();
    let result = start_crawl(&client, &config, &payload()).await;
    assert!(result.is_err());
}

// ── get_crawl_status ────────────────────────────────────────────────────

#[tokio::test]
async fn get_crawl_status_success() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/job-1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "success": true,
            "result": {
                "status": "completed",
                "records": [
                    {"url": "https://example.com", "html": "<h1>hi</h1>"}
                ],
                "total": 1
            },
            "errors": [],
            "messages": []
        })))
        .mount(&server)
        .await;

    let config = test_config(&server.uri());
    let client = Client::new();
    let result = get_crawl_status(&client, &config, "job-1").await.unwrap();
    assert_eq!(result.status, "completed");
    assert_eq!(result.records.len(), 1);
    assert_eq!(result.records[0].url, "https://example.com");
}

#[tokio::test]
async fn get_crawl_status_api_failure() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/job-bad"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "success": false,
            "result": null,
            "errors": [{"message": "not found"}],
            "messages": []
        })))
        .mount(&server)
        .await;

    let config = test_config(&server.uri());
    let client = Client::new();
    let result = get_crawl_status(&client, &config, "job-bad").await;
    assert!(result.is_err());
}

// ── cancel_crawl ────────────────────────────────────────────────────────

#[tokio::test]
async fn cancel_crawl_success() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/job-cancel"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "success": true,
            "result": null,
            "errors": [],
            "messages": []
        })))
        .mount(&server)
        .await;

    let config = test_config(&server.uri());
    let client = Client::new();
    cancel_crawl(&client, &config, "job-cancel").await.unwrap();
}

// ── poll_until_complete ─────────────────────────────────────────────────

#[tokio::test]
async fn poll_until_complete_immediate_success() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/job-poll"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "success": true,
            "result": {
                "status": "completed",
                "records": [],
                "total": 0
            },
            "errors": [],
            "messages": []
        })))
        .mount(&server)
        .await;

    let config = test_config(&server.uri());
    let client = Client::new();
    let result = poll_until_complete(&client, &config, "job-poll", 1, None)
        .await
        .unwrap();
    assert_eq!(result.status, "completed");
}

#[tokio::test]
async fn poll_until_complete_failure_status() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/job-fail"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "success": true,
            "result": {
                "status": "errored",
                "records": [],
                "total": 0
            },
            "errors": [],
            "messages": []
        })))
        .mount(&server)
        .await;

    let config = test_config(&server.uri());
    let client = Client::new();
    let result = poll_until_complete(&client, &config, "job-fail", 1, None).await;
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("errored"));
}

#[tokio::test]
async fn poll_with_status_callback() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/job-cb"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "success": true,
            "result": {
                "status": "completed",
                "records": [],
                "total": 0
            },
            "errors": [],
            "messages": []
        })))
        .mount(&server)
        .await;

    let config = test_config(&server.uri());
    let client = Client::new();

    let statuses = std::cell::RefCell::new(Vec::new());
    let cb = |status: &str, _: &browserflare::CrawlResult| {
        statuses.borrow_mut().push(status.to_string());
    };
    let result = poll_until_complete(&client, &config, "job-cb", 1, Some(&cb))
        .await
        .unwrap();
    assert_eq!(result.status, "completed");
    assert_eq!(*statuses.borrow(), vec!["completed"]);
}

// ── get_crawl_results_paginated ─────────────────────────────────────────

#[tokio::test]
async fn paginated_results_single_page() {
    let server = MockServer::start().await;
    // The paginated endpoint is GET /{job_id}?limit=...
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "success": true,
            "result": {
                "status": "completed",
                "records": [
                    {"url": "https://a.com"},
                    {"url": "https://b.com"}
                ],
                "total": 2,
                "cursor": null
            },
            "errors": [],
            "messages": []
        })))
        .mount(&server)
        .await;

    let config = test_config(&server.uri());
    let client = Client::new();
    let result = get_crawl_results_paginated(&client, &config, "job-page", 10, None, None)
        .await
        .unwrap();
    assert_eq!(result.records.len(), 2);
}

#[tokio::test]
async fn paginated_results_with_progress_callback() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "success": true,
            "result": {
                "status": "completed",
                "records": [{"url": "https://a.com"}],
                "total": 1,
                "cursor": null
            },
            "errors": [],
            "messages": []
        })))
        .mount(&server)
        .await;

    let config = test_config(&server.uri());
    let client = Client::new();

    let progress_calls = std::cell::RefCell::new(Vec::new());
    let cb = |fetched: usize, total: u64| {
        progress_calls.borrow_mut().push((fetched, total));
    };
    let result =
        get_crawl_results_paginated(&client, &config, "job-pg", 10, None, Some(&cb))
            .await
            .unwrap();
    assert_eq!(result.records.len(), 1);
    assert_eq!(*progress_calls.borrow(), vec![(1, 1)]);
}
