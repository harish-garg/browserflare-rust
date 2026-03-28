use reqwest::Client;

use crate::config::ApiConfig;
use crate::error::{BrowserflareError, Result};
use crate::payloads::{SnapshotPayload, SnapshotResult};

pub async fn take_snapshot(
    client: &Client,
    config: &ApiConfig,
    payload: &SnapshotPayload,
) -> Result<SnapshotResult> {
    let response = client
        .post(&config.base_url)
        .headers(config.headers.clone())
        .json(payload)
        .send()
        .await?;

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

    let result: SnapshotResult = response.json().await?;

    if !result.success {
        let error_msg = result
            .errors
            .as_ref()
            .map(|e| serde_json::to_value(e).unwrap_or_default())
            .unwrap_or_else(|| serde_json::json!({"error": "snapshot failed"}));
        return Err(BrowserflareError::ApiError(error_msg));
    }

    Ok(result)
}
