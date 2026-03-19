use reqwest::Client;

use crate::config::get_screenshot_api_config;
use crate::error::{BrowserflareError, Result};
use crate::payloads::{ScreenshotPayload, ScreenshotResult};

pub async fn take_screenshot(
    client: &Client,
    payload: &ScreenshotPayload,
) -> Result<ScreenshotResult> {
    let config = get_screenshot_api_config()?;
    let response = client
        .post(&config.base_url)
        .headers(config.headers)
        .json(payload)
        .send()
        .await?;

    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();

    if content_type.contains("application/json") {
        let data: serde_json::Value = response.json().await?;
        let error_msg = data
            .get("errors")
            .or_else(|| data.get("messages"))
            .cloned()
            .unwrap_or(data);
        return Err(BrowserflareError::ApiError(error_msg));
    }

    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        let truncated = match body.char_indices().nth(200) {
            Some((idx, _)) => &body[..idx],
            None => &body,
        };
        return Err(BrowserflareError::HttpError {
            status: status.as_u16(),
            body: truncated.to_string(),
        });
    }

    let bytes = response.bytes().await?.to_vec();
    Ok(ScreenshotResult {
        bytes,
        content_type,
    })
}
