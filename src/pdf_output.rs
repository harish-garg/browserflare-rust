use std::path::{Path, PathBuf};

use chrono::Local;
use serde_json::Value;

use crate::config::OUTPUT_DIR;
use crate::error::Result;
use crate::output::sanitize_filename;

const PDFS_DIR_NAME: &str = "pdfs";

fn pdfs_dir() -> PathBuf {
    Path::new(OUTPUT_DIR).join(PDFS_DIR_NAME)
}

fn pdf_log_path() -> PathBuf {
    pdfs_dir().join("pdf_log.json")
}

pub fn save_pdf(url: &str, pdf_bytes: &[u8], label: Option<&str>) -> Result<PathBuf> {
    let dir = pdfs_dir();
    std::fs::create_dir_all(&dir)?;

    let slug = sanitize_filename(url);
    let timestamp = Local::now().format("%Y%m%d_%H%M%S");
    let label_part = label
        .map(|l| format!("_{}", sanitize_filename(l)))
        .unwrap_or_default();
    let filename = format!("{slug}{label_part}_{timestamp}.pdf");

    let filepath = dir.join(&filename);
    std::fs::write(&filepath, pdf_bytes)?;

    Ok(filepath)
}

pub fn log_pdf(url: &str, filepath: &Path, payload: Option<&Value>) -> Result<()> {
    let dir = pdfs_dir();
    std::fs::create_dir_all(&dir)?;

    let log_path = pdf_log_path();
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
