use std::sync::Mutex;

use browserflare::payloads::{CrawlRecord, CrawlResult};
use browserflare::{
    diff_crawls, get_statistics, load_result, sanitize_filename, save_pdf, save_results,
    save_screenshot, search_results,
};
use serde_json::Map;
use tempfile::TempDir;

/// Global lock to serialize tests that call set_current_dir (process-global state).
static CWD_LOCK: Mutex<()> = Mutex::new(());

fn run_in_temp<F: FnOnce()>(f: F) {
    let _guard = CWD_LOCK.lock().unwrap();
    let tmp = TempDir::new().unwrap();
    let original = std::env::current_dir().unwrap();
    std::env::set_current_dir(tmp.path()).unwrap();
    f();
    std::env::set_current_dir(original).unwrap();
}

fn make_result(records: Vec<CrawlRecord>) -> CrawlResult {
    CrawlResult {
        status: "completed".into(),
        records,
        total: None,
        cursor: None,
        browser_seconds_used: Some(1.5),
        extra: Map::new(),
    }
}

fn make_record(url: &str, html: Option<&str>, markdown: Option<&str>) -> CrawlRecord {
    CrawlRecord {
        url: url.into(),
        status: Some("success".into()),
        html: html.map(|s| s.into()),
        markdown: markdown.map(|s| s.into()),
        json: None,
        extra: Map::new(),
    }
}

// ── sanitize_filename (pure, no cwd needed) ─────────────────────────────

#[test]
fn sanitize_strips_protocol() {
    assert_eq!(sanitize_filename("https://example.com"), "example.com");
    assert_eq!(sanitize_filename("http://example.com"), "example.com");
}

#[test]
fn sanitize_replaces_special_chars() {
    assert_eq!(
        sanitize_filename("https://example.com/a?b=1&c=2"),
        "example.com_a_b_1_c_2"
    );
}

#[test]
fn sanitize_empty_url() {
    assert_eq!(sanitize_filename(""), "page");
}

#[test]
fn sanitize_truncates_long_urls() {
    let long = format!("https://example.com/{}", "x".repeat(300));
    assert!(sanitize_filename(&long).len() <= 200);
}

// ── save_results + load_result ──────────────────────────────────────────

#[test]
fn save_and_load_results() {
    run_in_temp(|| {
        let result = make_result(vec![
            make_record("https://a.com", Some("<h1>A</h1>"), Some("# A")),
            make_record("https://b.com", Some("<h1>B</h1>"), None),
        ]);

        let path = save_results("test-job-1", &result, None).unwrap();
        assert!(path.exists());

        let loaded = load_result("test-job-1").unwrap().unwrap();
        assert_eq!(loaded.status, "completed");
        assert_eq!(loaded.records.len(), 2);
    });
}

#[test]
fn save_results_with_formats() {
    run_in_temp(|| {
        let result = make_result(vec![make_record(
            "https://example.com",
            Some("<h1>Hello</h1>"),
            Some("# Hello"),
        )]);

        let formats = vec!["html".to_string(), "markdown".to_string()];
        save_results("test-job-fmt", &result, Some(&formats)).unwrap();

        let pages_dir = std::env::current_dir()
            .unwrap()
            .join("output")
            .join("test-job-fmt")
            .join("pages");
        assert!(pages_dir.exists());

        let entries: Vec<_> = std::fs::read_dir(&pages_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .collect();
        let has_html = entries
            .iter()
            .any(|e| e.path().extension().map_or(false, |ext| ext == "html"));
        let has_md = entries
            .iter()
            .any(|e| e.path().extension().map_or(false, |ext| ext == "md"));
        assert!(has_html, "expected .html file in pages dir");
        assert!(has_md, "expected .md file in pages dir");

        let combined = std::env::current_dir()
            .unwrap()
            .join("output")
            .join("test-job-fmt")
            .join("combined.md");
        assert!(combined.exists());
    });
}

#[test]
fn load_result_missing_job() {
    run_in_temp(|| {
        let loaded = load_result("nonexistent-job").unwrap();
        assert!(loaded.is_none());
    });
}

// ── search_results ──────────────────────────────────────────────────────

#[test]
fn search_results_finds_matches() {
    run_in_temp(|| {
        let result = make_result(vec![
            make_record("https://a.com", None, Some("Rust is a systems language")),
            make_record("https://b.com", None, Some("Python is interpreted")),
        ]);
        save_results("search-job", &result, None).unwrap();

        let matches = search_results("search-job", "Rust").unwrap();
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].url, "https://a.com");
        assert!(matches[0].snippet.contains("Rust"));
    });
}

#[test]
fn search_results_case_insensitive() {
    run_in_temp(|| {
        let result = make_result(vec![make_record(
            "https://a.com",
            None,
            Some("Hello World"),
        )]);
        save_results("search-ci", &result, None).unwrap();

        let matches = search_results("search-ci", "hello").unwrap();
        assert_eq!(matches.len(), 1);
    });
}

// ── get_statistics ──────────────────────────────────────────────────────

#[test]
fn statistics_returns_correct_counts() {
    run_in_temp(|| {
        let result = make_result(vec![
            make_record("https://a.com", Some("<h1>A</h1>"), Some("# A")),
            make_record("https://b.com", Some("<h1>B</h1>"), None),
        ]);
        save_results("stats-job", &result, None).unwrap();

        let stats = get_statistics("stats-job").unwrap().unwrap();
        assert_eq!(stats.job_status, "completed");
        assert_eq!(stats.total_records, 2);
        assert_eq!(stats.browser_seconds, 1.5);
        assert!(stats.total_content_size_bytes > 0);
        assert!(stats.status_breakdown.contains_key("success"));
        assert_eq!(*stats.status_breakdown.get("success").unwrap(), 2);
    });
}

// ── diff_crawls ─────────────────────────────────────────────────────────

#[test]
fn diff_crawls_detects_added_and_removed() {
    run_in_temp(|| {
        let result_a = make_result(vec![
            make_record("https://a.com", None, None),
            make_record("https://b.com", None, None),
        ]);
        let result_b = make_result(vec![
            make_record("https://b.com", None, None),
            make_record("https://c.com", None, None),
        ]);

        save_results("diff-a", &result_a, None).unwrap();
        save_results("diff-b", &result_b, None).unwrap();

        let diff = diff_crawls("diff-a", "diff-b").unwrap().unwrap();
        assert_eq!(diff.added, vec!["https://c.com"]);
        assert_eq!(diff.removed, vec!["https://a.com"]);
        assert_eq!(diff.common, vec!["https://b.com"]);
        assert_eq!(diff.count_a, 2);
        assert_eq!(diff.count_b, 2);
    });
}

// ── save_screenshot ─────────────────────────────────────────────────────

#[test]
fn save_screenshot_creates_file() {
    run_in_temp(|| {
        let fake_png = vec![0x89, 0x50, 0x4E, 0x47];
        let path =
            save_screenshot("https://example.com", &fake_png, "image/png", None).unwrap();
        assert!(path.exists());
        assert_eq!(std::fs::read(&path).unwrap(), fake_png);
        assert!(path.to_string_lossy().ends_with(".png"));
    });
}

#[test]
fn save_screenshot_jpeg_extension() {
    run_in_temp(|| {
        let path = save_screenshot("https://example.com", b"fake", "image/jpeg", Some("label"))
            .unwrap();
        assert!(path.exists());
        assert!(path.to_string_lossy().ends_with(".jpeg"));
    });
}

// ── save_pdf ────────────────────────────────────────────────────────────

#[test]
fn save_pdf_creates_file() {
    run_in_temp(|| {
        let fake_pdf = b"%PDF-1.4 test";
        let path = save_pdf("https://example.com", fake_pdf, None).unwrap();
        assert!(path.exists());
        assert_eq!(std::fs::read(&path).unwrap(), fake_pdf);
        assert!(path.to_string_lossy().ends_with(".pdf"));
    });
}

#[test]
fn save_pdf_with_label() {
    run_in_temp(|| {
        let path = save_pdf("https://example.com", b"fake", Some("invoice")).unwrap();
        assert!(path.exists());
        assert!(path.to_string_lossy().contains("invoice"));
    });
}
