use serde::{Deserialize, Deserializer, Serialize};
use serde_json::{Map, Value};

// ── Shared sub-structs ──────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Viewport {
    pub width: u32,
    pub height: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_scale_factor: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct WaitForSelector {
    pub selector: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct GotoOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wait_until: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct JsonOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PdfMargin {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bottom: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub left: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub right: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PdfOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub landscape: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub print_background: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scale: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_header_footer: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub header_template: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub footer_template: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub margin: Option<PdfMargin>,
}

// ── Crawl payload ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CrawlPayload {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub formats: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reject_resource_types: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub depth: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub render: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_external_links: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_subdomains: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_patterns: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude_patterns: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_age: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified_since: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wait_for_selector: Option<WaitForSelector>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub json_options: Option<JsonOptions>,
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub extra: Option<Map<String, Value>>,
}

// ── Screenshot payload ──────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ScreenshotOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub full_page: Option<bool>,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quality: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub omit_background: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ScreenshotPayload {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub viewport: Option<Viewport>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub screenshot_options: Option<ScreenshotOptions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selector: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wait_for_selector: Option<WaitForSelector>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub goto_options: Option<GotoOptions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,
}

// ── PDF payload ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PdfPayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub html: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub viewport: Option<Viewport>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pdf_options: Option<PdfOptions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wait_for_selector: Option<WaitForSelector>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub goto_options: Option<GotoOptions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,
}

// ── Content payload ────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ContentPayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub html: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub viewport: Option<Viewport>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub goto_options: Option<GotoOptions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wait_for_selector: Option<WaitForSelector>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reject_resource_types: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reject_request_pattern: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_resource_types: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_request_pattern: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub set_extra_http_headers: Option<Map<String, Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cookies: Option<Vec<Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub set_javascript_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub emulate_media_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub best_attempt: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action_timeout: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wait_for_timeout: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub add_script_tag: Option<Vec<Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub add_style_tag: Option<Vec<Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ContentMeta {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentResult {
    pub success: bool,
    pub result: Option<String>,
    pub errors: Option<Vec<Value>>,
    pub meta: Option<ContentMeta>,
}

// ── Helpers ─────────────────────────────────────────────────────────────

fn deserialize_cursor<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let value: Option<Value> = Option::deserialize(deserializer)?;
    Ok(value.and_then(|v| match v {
        Value::String(s) if s.is_empty() => None,
        Value::String(s) => Some(s),
        Value::Number(n) => Some(n.to_string()),
        Value::Null => None,
        other => Some(other.to_string()),
    }))
}

// ── Response types ──────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CfApiResponse<T> {
    pub success: bool,
    pub result: Option<T>,
    pub errors: Option<Vec<Value>>,
    pub messages: Option<Vec<Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CrawlResult {
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub records: Vec<CrawlRecord>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<u64>,
    #[serde(default, deserialize_with = "deserialize_cursor", skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub browser_seconds_used: Option<f64>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CrawlRecord {
    #[serde(default)]
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub html: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub markdown: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub json: Option<Value>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug)]
pub struct ScreenshotResult {
    pub bytes: Vec<u8>,
    pub content_type: String,
}

#[derive(Debug)]
pub struct PdfResult {
    pub bytes: Vec<u8>,
    pub content_type: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crawl_payload_serializes_with_defaults() {
        let payload = CrawlPayload {
            url: "https://example.com".into(),
            limit: Some(100),
            ..Default::default()
        };
        let json = serde_json::to_value(&payload).unwrap();
        assert_eq!(json["url"], "https://example.com");
        assert_eq!(json["limit"], 100);
        assert!(json.get("formats").is_none());
        assert!(json.get("depth").is_none());
    }

    #[test]
    fn screenshot_payload_nests_options_under_screenshot_options() {
        let payload = ScreenshotPayload {
            url: "https://example.com".into(),
            screenshot_options: Some(ScreenshotOptions {
                format: Some("jpeg".into()),
                full_page: Some(true),
                quality: Some(80),
                omit_background: None,
            }),
            ..Default::default()
        };
        let json = serde_json::to_value(&payload).unwrap();
        assert_eq!(json["screenshotOptions"]["type"], "jpeg");
        assert_eq!(json["screenshotOptions"]["fullPage"], true);
        assert_eq!(json["screenshotOptions"]["quality"], 80);
        assert!(json.get("fullPage").is_none());
        assert!(json.get("type").is_none());
        assert!(json.get("quality").is_none());
    }

    #[test]
    fn pdf_payload_skips_none_fields() {
        let payload = PdfPayload {
            url: Some("https://example.com".into()),
            ..Default::default()
        };
        let json = serde_json::to_value(&payload).unwrap();
        assert_eq!(json["url"], "https://example.com");
        assert!(json.get("html").is_none());
        assert!(json.get("pdfOptions").is_none());
    }

    #[test]
    fn crawl_payload_with_extra_fields() {
        let mut extra = Map::new();
        extra.insert("customField".into(), Value::Bool(true));
        let payload = CrawlPayload {
            url: "https://example.com".into(),
            extra: Some(extra),
            ..Default::default()
        };
        let json = serde_json::to_value(&payload).unwrap();
        assert_eq!(json["customField"], true);
    }
}
