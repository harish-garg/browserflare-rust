use std::time::Duration;

use reqwest::Client;
use serde_json::Value;

use crate::config::{ApiConfig, FAILURE_STATUSES, TERMINAL_STATUSES};
use crate::error::{BrowserflareError, Result};
use crate::payloads::{CfApiResponse, CrawlPayload, CrawlResult};

type StatusCallback<'a> = Option<&'a dyn Fn(&str, &CrawlResult)>;

pub async fn start_crawl(
    client: &Client,
    config: &ApiConfig,
    payload: &CrawlPayload,
) -> Result<String> {
    let response = client
        .post(&config.base_url)
        .headers(config.headers.clone())
        .json(payload)
        .send()
        .await?;

    let status = response.status();
    let data: CfApiResponse<Value> = response.json().await?;

    if !status.is_success() || !data.success {
        return Err(BrowserflareError::ApiError(
            serde_json::to_value(&data).unwrap_or_default(),
        ));
    }

    match data.result {
        Some(Value::String(job_id)) => Ok(job_id),
        Some(other) => Ok(other.to_string()),
        None => Err(BrowserflareError::ApiError(
            serde_json::json!({"error": "no result in response"}),
        )),
    }
}

pub async fn get_crawl_status(
    client: &Client,
    config: &ApiConfig,
    job_id: &str,
) -> Result<CrawlResult> {
    let response = client
        .get(format!("{}/{job_id}", config.base_url))
        .headers(config.headers.clone())
        .send()
        .await?;

    let data: CfApiResponse<CrawlResult> = response.json().await?;

    if !data.success {
        return Err(BrowserflareError::ApiError(
            serde_json::to_value(&data).unwrap_or_default(),
        ));
    }

    data.result.ok_or_else(|| {
        BrowserflareError::ApiError(serde_json::json!({"error": "no result in response"}))
    })
}

pub async fn get_crawl_results_paginated(
    client: &Client,
    config: &ApiConfig,
    job_id: &str,
    limit_per_page: u32,
    status_filter: Option<&str>,
    on_progress: Option<&dyn Fn(usize, u64)>,
) -> Result<CrawlResult> {
    let mut all_records = Vec::new();
    let mut cursor: Option<String> = None;
    let mut total: Option<u64> = None;
    let mut last_result: Option<CrawlResult>;

    loop {
        let mut url = format!("{}/{}?limit={}", config.base_url, job_id, limit_per_page);
        if let Some(ref c) = cursor {
            url.push_str(&format!("&cursor={c}"));
        }
        if let Some(sf) = status_filter {
            url.push_str(&format!("&status={sf}"));
        }

        let response = client
            .get(&url)
            .headers(config.headers.clone())
            .send()
            .await?;

        let data: CfApiResponse<CrawlResult> = response.json().await?;

        if !data.success {
            return Err(BrowserflareError::ApiError(
                serde_json::to_value(&data).unwrap_or_default(),
            ));
        }

        let result = data.result.ok_or_else(|| {
            BrowserflareError::ApiError(serde_json::json!({"error": "no result in response"}))
        })?;

        all_records.extend(result.records.clone());

        if total.is_none() {
            total = Some(result.total.unwrap_or(all_records.len() as u64));
        }

        if let (Some(t), Some(cb)) = (total, on_progress) {
            if t > 0 {
                cb(all_records.len(), t);
            }
        }

        let next_cursor = result.cursor.clone();
        let had_records = !result.records.is_empty();
        last_result = Some(result);

        match next_cursor {
            Some(c) if !c.is_empty() && had_records => cursor = Some(c),
            _ => break,
        }
    }

    let mut result = last_result.unwrap();
    result.records = all_records;
    Ok(result)
}

pub async fn cancel_crawl(
    client: &Client,
    config: &ApiConfig,
    job_id: &str,
) -> Result<()> {
    let response = client
        .delete(format!("{}/{job_id}", config.base_url))
        .headers(config.headers.clone())
        .send()
        .await?;

    let data: CfApiResponse<Value> = response.json().await?;

    if !data.success {
        return Err(BrowserflareError::ApiError(
            serde_json::to_value(&data).unwrap_or_default(),
        ));
    }

    Ok(())
}

pub async fn poll_until_complete(
    client: &Client,
    config: &ApiConfig,
    job_id: &str,
    interval_secs: u64,
    on_status: StatusCallback<'_>,
) -> Result<CrawlResult> {
    loop {
        let result = get_crawl_status(client, config, job_id).await?;
        let status = result.status.as_str();

        if let Some(cb) = on_status {
            cb(status, &result);
        }

        if TERMINAL_STATUSES.contains(status) {
            if FAILURE_STATUSES.contains(status) {
                return Err(BrowserflareError::CrawlFailed {
                    status: status.to_string(),
                    result: serde_json::to_value(&result).unwrap_or_default(),
                });
            }
            return Ok(result);
        }

        tokio::time::sleep(Duration::from_secs(interval_secs)).await;
    }
}
