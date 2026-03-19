use std::path::{Path, PathBuf};

use chrono::Local;
use serde_json::Value;

use crate::config::OUTPUT_DIR;
use crate::error::Result;
use crate::output::sanitize_filename;

const SCREENSHOTS_DIR_NAME: &str = "screenshots";

fn screenshots_dir() -> PathBuf {
    Path::new(OUTPUT_DIR).join(SCREENSHOTS_DIR_NAME)
}

fn screenshot_log_path() -> PathBuf {
    screenshots_dir().join("screenshot_log.json")
}

pub fn save_screenshot(
    url: &str,
    image_bytes: &[u8],
    content_type: &str,
    label: Option<&str>,
) -> Result<PathBuf> {
    let dir = screenshots_dir();
    std::fs::create_dir_all(&dir)?;

    let ext = match content_type {
        "image/jpeg" => "jpeg",
        "image/webp" => "webp",
        _ => "png",
    };

    let slug = sanitize_filename(url);
    let timestamp = Local::now().format("%Y%m%d_%H%M%S");
    let label_part = label
        .map(|l| format!("_{}", sanitize_filename(l)))
        .unwrap_or_default();
    let filename = format!("{slug}{label_part}_{timestamp}.{ext}");

    let filepath = dir.join(&filename);
    std::fs::write(&filepath, image_bytes)?;

    Ok(filepath)
}

pub fn log_screenshot(url: &str, filepath: &Path, payload: Option<&Value>) -> Result<()> {
    let dir = screenshots_dir();
    std::fs::create_dir_all(&dir)?;

    let log_path = screenshot_log_path();
    let mut entries: Vec<Value> = if log_path.exists() {
        let content = std::fs::read_to_string(&log_path)?;
        serde_json::from_str(&content)?
    } else {
        Vec::new()
    };

    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    entries.push(serde_json::json!({
        "url": url,
        "filepath": filepath.to_string_lossy(),
        "timestamp": timestamp,
        "payload": payload,
    }));

    let json_str = serde_json::to_string_pretty(&entries)?;
    std::fs::write(&log_path, json_str)?;

    Ok(())
}
