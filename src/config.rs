use std::collections::HashSet;
use std::sync::LazyLock;

use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};

use crate::error::{BrowserflareError, Result};

pub const JOBS_FILE: &str = "crawl_jobs.json";
pub const OUTPUT_DIR: &str = "output";

pub static TERMINAL_STATUSES: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
    HashSet::from([
        "completed",
        "cancelled_due_to_timeout",
        "cancelled_due_to_limits",
        "cancelled_by_user",
        "errored",
        "error",
        "failed",
    ])
});

pub static SUCCESS_STATUSES: LazyLock<HashSet<&'static str>> =
    LazyLock::new(|| HashSet::from(["completed"]));

pub static FAILURE_STATUSES: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
    TERMINAL_STATUSES
        .difference(&SUCCESS_STATUSES)
        .copied()
        .collect()
});

pub const RESOURCE_TYPES: &[&str] = &[
    "image",
    "media",
    "font",
    "stylesheet",
    "script",
    "xhr",
    "fetch",
    "websocket",
    "eventsource",
    "manifest",
    "texttrack",
    "other",
];

pub const DEFAULT_REJECT_RESOURCES: &[&str] = &["image", "media", "font", "stylesheet"];

pub const CRAWL_SOURCES: &[&str] = &["all", "sitemaps", "links"];

pub const OUTPUT_FORMATS: &[&str] = &["html", "markdown", "json"];

pub const SCREENSHOT_FORMATS: &[&str] = &["png", "jpeg", "webp"];

pub const PDF_PAGE_FORMATS: &[&str] = &[
    "letter", "legal", "tabloid", "ledger", "a0", "a1", "a2", "a3", "a4", "a5", "a6",
];

pub const WAIT_UNTIL_OPTIONS: &[&str] = &["load", "domcontentloaded", "networkidle0", "networkidle2"];

pub struct ApiConfig {
    pub base_url: String,
    pub headers: HeaderMap,
}

fn get_credentials() -> Result<(String, String)> {
    let _ = dotenvy::dotenv();
    let account_id = std::env::var("CF_ACCOUNT_ID").unwrap_or_default();
    let api_token = std::env::var("CF_API_TOKEN").unwrap_or_default();

    if account_id.is_empty() || api_token.is_empty() {
        return Err(BrowserflareError::MissingCredentials);
    }

    Ok((account_id, api_token))
}

fn build_headers(api_token: &str) -> Result<HeaderMap> {
    let mut headers = HeaderMap::new();
    let auth_value = HeaderValue::from_str(&format!("Bearer {api_token}"))
        .map_err(|_| BrowserflareError::MissingCredentials)?;
    headers.insert(AUTHORIZATION, auth_value);
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    Ok(headers)
}

pub fn get_api_config() -> Result<ApiConfig> {
    let (account_id, api_token) = get_credentials()?;
    Ok(ApiConfig {
        base_url: format!(
            "https://api.cloudflare.com/client/v4/accounts/{account_id}/browser-rendering/crawl"
        ),
        headers: build_headers(&api_token)?,
    })
}

pub fn get_screenshot_api_config() -> Result<ApiConfig> {
    let (account_id, api_token) = get_credentials()?;
    Ok(ApiConfig {
        base_url: format!(
            "https://api.cloudflare.com/client/v4/accounts/{account_id}/browser-rendering/screenshot"
        ),
        headers: build_headers(&api_token)?,
    })
}

pub fn get_pdf_api_config() -> Result<ApiConfig> {
    let (account_id, api_token) = get_credentials()?;
    Ok(ApiConfig {
        base_url: format!(
            "https://api.cloudflare.com/client/v4/accounts/{account_id}/browser-rendering/pdf"
        ),
        headers: build_headers(&api_token)?,
    })
}

/// Build an `ApiConfig` pointing at an arbitrary base URL with a dummy bearer token.
/// Useful for testing against a mock HTTP server.
pub fn test_config(base_url: &str) -> ApiConfig {
    ApiConfig {
        base_url: base_url.to_string(),
        headers: build_headers("test-token").expect("valid test headers"),
    }
}
