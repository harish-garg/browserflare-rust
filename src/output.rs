use std::collections::HashMap;
use std::path::{Path, PathBuf};

use chrono::Local;
use regex::Regex;
use serde::Serialize;
use serde_json::Value;

use crate::config::OUTPUT_DIR;
use crate::error::Result;
use crate::payloads::CrawlResult;

pub fn sanitize_filename(url: &str) -> String {
    let re_proto = Regex::new(r"https?://").unwrap();
    let re_nonword = Regex::new(r"[^\w\-.]").unwrap();
    let re_multi_underscore = Regex::new(r"_+").unwrap();

    let name = re_proto.replace_all(url, "");
    let name = re_nonword.replace_all(&name, "_");
    let name = re_multi_underscore.replace_all(&name, "_");
    let name = name.trim_matches('_');

    if name.is_empty() {
        "page".to_string()
    } else {
        name.chars().take(200).collect()
    }
}

pub fn find_result_path(job_id: &str) -> Option<PathBuf> {
    // New pattern: output/crawls/{slug}_{label}_{timestamp}/raw.json
    if let Ok(Some(job)) = crate::jobs::find_job(job_id) {
        if let Some(dir_name) = job.extra.get("output_dir").and_then(|v| v.as_str()) {
            let path = Path::new(OUTPUT_DIR).join("crawls").join(dir_name).join("raw.json");
            if path.exists() {
                return Some(path);
            }
        }
    }

    // Legacy: output/{job_id}/raw.json
    let old_path = Path::new(OUTPUT_DIR).join(job_id).join("raw.json");
    if old_path.exists() {
        return Some(old_path);
    }

    // Even older: output/{job_id}.json
    let oldest_path = Path::new(OUTPUT_DIR).join(format!("{job_id}.json"));
    if oldest_path.exists() {
        return Some(oldest_path);
    }

    None
}

pub fn load_result(job_id: &str) -> Result<Option<CrawlResult>> {
    let Some(path) = find_result_path(job_id) else {
        return Ok(None);
    };
    let content = std::fs::read_to_string(path)?;
    let result: CrawlResult = serde_json::from_str(&content)?;
    Ok(Some(result))
}

pub fn save_results(
    job_id: &str,
    url: &str,
    label: Option<&str>,
    result: &CrawlResult,
    formats: Option<&[String]>,
) -> Result<PathBuf> {
    let slug = sanitize_filename(url);
    let timestamp = Local::now().format("%Y%m%d_%H%M%S");
    let label_part = label
        .map(|l| format!("_{}", sanitize_filename(l)))
        .unwrap_or_default();
    let dir_name = format!("{slug}{label_part}_{timestamp}");

    let crawls_dir = Path::new(OUTPUT_DIR).join("crawls");
    let job_dir = crawls_dir.join(&dir_name);
    std::fs::create_dir_all(&job_dir)?;

    // Store output_dir in job record for lookup
    let mut fields = serde_json::Map::new();
    fields.insert("output_dir".into(), Value::String(dir_name));
    let _ = crate::jobs::update_job(job_id, fields);

    let raw_path = job_dir.join("raw.json");
    let json_str = serde_json::to_string_pretty(result)?;
    std::fs::write(&raw_path, &json_str)?;

    let records = &result.records;
    let formats = match formats {
        Some(f) if !f.is_empty() && !records.is_empty() => f,
        _ => return Ok(raw_path),
    };

    let pages_dir = job_dir.join("pages");
    std::fs::create_dir_all(&pages_dir)?;

    let mut all_markdown = Vec::new();
    let mut all_json_data: Vec<Value> = Vec::new();

    for record in records {
        let url = &record.url;
        let slug = sanitize_filename(url);

        if formats.iter().any(|f| f == "html") {
            if let Some(ref html) = record.html {
                let path = pages_dir.join(format!("{slug}.html"));
                std::fs::write(&path, html)?;
            }
        }

        if formats.iter().any(|f| f == "markdown") {
            if let Some(ref md) = record.markdown {
                let path = pages_dir.join(format!("{slug}.md"));
                std::fs::write(&path, md)?;
                all_markdown.push(format!("# {url}\n\n{md}\n\n---\n\n"));
            }
        }

        if formats.iter().any(|f| f == "json") {
            if let Some(ref json_val) = record.json {
                let path = pages_dir.join(format!("{slug}.json"));
                let s = serde_json::to_string_pretty(json_val)?;
                std::fs::write(&path, s)?;
                all_json_data.push(serde_json::json!({"url": url, "data": json_val}));
            }
        }
    }

    if !all_markdown.is_empty() {
        let combined = job_dir.join("combined.md");
        std::fs::write(&combined, all_markdown.join(""))?;
    }

    if !all_json_data.is_empty() {
        let combined = job_dir.join("combined.json");
        let s = serde_json::to_string_pretty(&all_json_data)?;
        std::fs::write(&combined, s)?;
    }

    Ok(raw_path)
}

#[derive(Debug, Clone, Serialize)]
pub struct SearchMatch {
    pub url: String,
    pub snippet: String,
}

pub fn search_results(job_id: &str, query: &str) -> Result<Vec<SearchMatch>> {
    let Some(result) = load_result(job_id)? else {
        return Ok(vec![]);
    };

    let query_lower = query.to_lowercase();
    let mut matches = Vec::new();

    for record in &result.records {
        let url = &record.url;
        let searchable = record
            .markdown
            .as_deref()
            .or(record.html.as_deref())
            .unwrap_or("");

        let url_match = url.to_lowercase().contains(&query_lower);
        let content_match = searchable.to_lowercase().contains(&query_lower);

        if url_match || content_match {
            let snippet = {
                let lower = searchable.to_lowercase();
                if let Some(byte_idx) = lower.find(&query_lower) {
                    // Convert byte index in lowercased string to char index
                    let match_char_idx = lower[..byte_idx].chars().count();
                    let match_char_len = query_lower.chars().count();

                    let chars: Vec<(usize, char)> = searchable.char_indices().collect();
                    let total_chars = chars.len();

                    let start_char = match_char_idx.saturating_sub(80);
                    let end_char = (match_char_idx + match_char_len + 80).min(total_chars);

                    let start_byte = chars[start_char].0;
                    let end_byte = if end_char >= total_chars {
                        searchable.len()
                    } else {
                        chars[end_char].0
                    };

                    let mut s = searchable[start_byte..end_byte].replace('\n', " ");
                    s = s.trim().to_string();
                    if start_char > 0 {
                        s = format!("...{s}");
                    }
                    if end_char < total_chars {
                        s = format!("{s}...");
                    }
                    s
                } else {
                    String::new()
                }
            };

            matches.push(SearchMatch {
                url: url.clone(),
                snippet,
            });
        }
    }

    Ok(matches)
}

#[derive(Debug, Clone, Serialize)]
pub struct CrawlStatistics {
    pub job_status: String,
    pub total_records: usize,
    pub status_breakdown: HashMap<String, usize>,
    pub total_content_size_bytes: usize,
    pub total_content_size_mb: f64,
    pub browser_seconds: f64,
}

pub fn get_statistics(job_id: &str) -> Result<Option<CrawlStatistics>> {
    let Some(result) = load_result(job_id)? else {
        return Ok(None);
    };

    let records = &result.records;
    let mut status_counts: HashMap<String, usize> = HashMap::new();
    let mut total_size: usize = 0;

    for record in records {
        let status = record
            .status
            .as_deref()
            .unwrap_or("unknown")
            .to_string();
        *status_counts.entry(status).or_insert(0) += 1;

        if let Some(ref html) = record.html {
            total_size += html.len();
        }
        if let Some(ref md) = record.markdown {
            total_size += md.len();
        }
    }

    Ok(Some(CrawlStatistics {
        job_status: result.status.clone(),
        total_records: records.len(),
        status_breakdown: status_counts,
        total_content_size_bytes: total_size,
        total_content_size_mb: (total_size as f64 / (1024.0 * 1024.0) * 100.0).round() / 100.0,
        browser_seconds: result.browser_seconds_used.unwrap_or(0.0),
    }))
}

#[derive(Debug, Clone, Serialize)]
pub struct CrawlDiff {
    pub added: Vec<String>,
    pub removed: Vec<String>,
    pub common: Vec<String>,
    pub count_a: usize,
    pub count_b: usize,
}

pub fn diff_crawls(job_id_a: &str, job_id_b: &str) -> Result<Option<CrawlDiff>> {
    let result_a = load_result(job_id_a)?;
    let result_b = load_result(job_id_b)?;

    let (Some(result_a), Some(result_b)) = (result_a, result_b) else {
        return Ok(None);
    };

    let urls_a: std::collections::HashSet<String> =
        result_a.records.iter().map(|r| r.url.clone()).collect();
    let urls_b: std::collections::HashSet<String> =
        result_b.records.iter().map(|r| r.url.clone()).collect();

    let mut added: Vec<String> = urls_b.difference(&urls_a).cloned().collect();
    let mut removed: Vec<String> = urls_a.difference(&urls_b).cloned().collect();
    let mut common: Vec<String> = urls_a.intersection(&urls_b).cloned().collect();

    added.sort();
    removed.sort();
    common.sort();

    Ok(Some(CrawlDiff {
        count_a: urls_a.len(),
        count_b: urls_b.len(),
        added,
        removed,
        common,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("https://example.com/page"), "example.com_page");
        assert_eq!(sanitize_filename("http://foo.bar/a?b=1&c=2"), "foo.bar_a_b_1_c_2");
        assert_eq!(sanitize_filename(""), "page");
    }

    #[test]
    fn test_sanitize_filename_long_url() {
        let long_url = format!("https://example.com/{}", "a".repeat(300));
        let result = sanitize_filename(&long_url);
        assert!(result.len() <= 200);
    }
}
