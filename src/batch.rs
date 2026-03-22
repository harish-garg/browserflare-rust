use std::path::Path;

use chrono::Local;
use reqwest::Client;
use serde_json::{Map, Value};

use crate::api::{get_crawl_results_paginated, poll_until_complete, start_crawl};
use crate::config::ApiConfig;
use crate::error::{BrowserflareError, Result};
use crate::jobs::{add_job, update_job};
use crate::output::save_results;
use crate::payloads::CrawlPayload;

#[derive(Debug)]
pub enum BatchEvent {
    NoUrls,
    UrlsFound { count: usize },
    CrawlStart { index: usize, total: usize, url: String },
    CrawlFailed { index: usize, total: usize, url: String, error: BrowserflareError },
    CrawlSubmitted { index: usize, total: usize, url: String, job_id: String },
    CrawlWaiting { job_id: String },
    CrawlEnded { job_id: String, status: String },
    CrawlComplete { job_id: String, page_count: usize },
    BatchDone { started: usize, total: usize },
}

pub fn load_urls(file_path: &Path) -> Result<Vec<String>> {
    let content = std::fs::read_to_string(file_path)?;
    let content = content.trim();

    // Try JSON array first
    if let Ok(Value::Array(arr)) = serde_json::from_str::<Value>(content) {
        let urls: Vec<String> = arr
            .into_iter()
            .filter_map(|v| {
                if let Value::String(s) = v {
                    let trimmed = s.trim().to_string();
                    if trimmed.is_empty() { None } else { Some(trimmed) }
                } else {
                    None
                }
            })
            .collect();
        return Ok(urls);
    }

    // Fall back to line-by-line
    let urls: Vec<String> = content
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .collect();

    Ok(urls)
}

pub async fn run_batch(
    client: &Client,
    config: &ApiConfig,
    urls_file: &Path,
    base_payload: &CrawlPayload,
    wait: bool,
    formats: Option<&[String]>,
    on_event: Option<&dyn Fn(BatchEvent)>,
) -> Result<Vec<String>> {
    let urls = load_urls(urls_file)?;

    if urls.is_empty() {
        if let Some(cb) = on_event {
            cb(BatchEvent::NoUrls);
        }
        return Ok(vec![]);
    }

    if let Some(cb) = on_event {
        cb(BatchEvent::UrlsFound { count: urls.len() });
    }

    let total = urls.len();
    let mut job_ids = Vec::new();

    for (i, url) in urls.iter().enumerate() {
        let index = i + 1;
        let mut payload = base_payload.clone();
        payload.url = url.clone();

        if let Some(cb) = on_event {
            cb(BatchEvent::CrawlStart {
                index,
                total,
                url: url.clone(),
            });
        }

        let job_id = match start_crawl(client, config, &payload).await {
            Ok(id) => id,
            Err(err) => {
                if let Some(cb) = on_event {
                    cb(BatchEvent::CrawlFailed {
                        index,
                        total,
                        url: url.clone(),
                        error: err,
                    });
                }
                continue;
            }
        };

        if let Some(cb) = on_event {
            cb(BatchEvent::CrawlSubmitted {
                index,
                total,
                url: url.clone(),
                job_id: job_id.clone(),
            });
        }

        let payload_value = serde_json::to_value(&payload).unwrap_or_default();
        let label = format!("batch {index}/{total}");
        add_job(&job_id, url, &payload_value, Some(&label))?;
        job_ids.push(job_id.clone());

        if wait {
            if let Some(cb) = on_event {
                cb(BatchEvent::CrawlWaiting {
                    job_id: job_id.clone(),
                });
            }

            match poll_until_complete(client, config, &job_id, 3, None).await {
                Err(BrowserflareError::CrawlFailed { status, .. }) => {
                    if let Some(cb) = on_event {
                        cb(BatchEvent::CrawlEnded {
                            job_id: job_id.clone(),
                            status: status.clone(),
                        });
                    }
                    let mut fields = Map::new();
                    fields.insert("status".into(), Value::String(status));
                    update_job(&job_id, fields)?;
                }
                Err(err) => {
                    if let Some(cb) = on_event {
                        cb(BatchEvent::CrawlEnded {
                            job_id: job_id.clone(),
                            status: "failed".to_string(),
                        });
                    }
                    let mut fields = Map::new();
                    fields.insert("status".into(), Value::String("failed".into()));
                    fields.insert("error".into(), Value::String(err.to_string()));
                    update_job(&job_id, fields)?;
                }
                Ok(mut result) => {
                    let page_count = result.records.len();
                    let result_total = result.total.unwrap_or(page_count as u64);

                    if result_total > page_count as u64 {
                        if let Ok(full) =
                            get_crawl_results_paginated(client, config, &job_id, 100, None, None).await
                        {
                            result = full;
                        }
                    }

                    let page_count = result.records.len();
                    save_results(&job_id, &result, formats)?;

                    let mut fields = Map::new();
                    fields.insert("status".into(), Value::String("completed".into()));
                    fields.insert(
                        "completed_at".into(),
                        Value::String(Local::now().format("%Y-%m-%d %H:%M:%S").to_string()),
                    );
                    fields.insert(
                        "page_count".into(),
                        Value::Number(serde_json::Number::from(page_count)),
                    );
                    update_job(&job_id, fields)?;

                    if let Some(cb) = on_event {
                        cb(BatchEvent::CrawlComplete {
                            job_id: job_id.clone(),
                            page_count,
                        });
                    }
                }
            }
        }
    }

    if let Some(cb) = on_event {
        cb(BatchEvent::BatchDone {
            started: job_ids.len(),
            total,
        });
    }

    Ok(job_ids)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_load_urls_json_array() {
        let dir = std::env::temp_dir();
        let path = dir.join("test_urls.json");
        let mut f = std::fs::File::create(&path).unwrap();
        writeln!(f, r#"["https://a.com", "https://b.com", ""]"#).unwrap();

        let urls = load_urls(&path).unwrap();
        assert_eq!(urls, vec!["https://a.com", "https://b.com"]);

        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_load_urls_line_delimited() {
        let dir = std::env::temp_dir();
        let path = dir.join("test_urls.txt");
        let mut f = std::fs::File::create(&path).unwrap();
        writeln!(f, "# comment\nhttps://a.com\n\nhttps://b.com").unwrap();

        let urls = load_urls(&path).unwrap();
        assert_eq!(urls, vec!["https://a.com", "https://b.com"]);

        std::fs::remove_file(&path).ok();
    }
}
