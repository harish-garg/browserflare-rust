use std::sync::Mutex;

use browserflare::{add_job, delete_jobs, find_job, get_jobs_by_status, load_jobs, update_job};
use serde_json::{json, Map, Value};
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

#[test]
fn load_jobs_empty_when_no_file() {
    run_in_temp(|| {
        let jobs = load_jobs().unwrap();
        assert!(jobs.is_empty());
    });
}

#[test]
fn add_and_find_job() {
    run_in_temp(|| {
        let payload = json!({"url": "https://example.com"});
        let job = add_job("job-1", "https://example.com", &payload, Some("test")).unwrap();
        assert_eq!(job.job_id, "job-1");
        assert_eq!(job.url, "https://example.com");
        assert_eq!(job.status, "pending");
        assert_eq!(job.label, Some("test".into()));

        let found = find_job("job-1").unwrap().unwrap();
        assert_eq!(found.job_id, "job-1");

        let not_found = find_job("nonexistent").unwrap();
        assert!(not_found.is_none());
    });
}

#[test]
fn update_job_status() {
    run_in_temp(|| {
        let payload = json!({});
        add_job("job-u", "https://example.com", &payload, None).unwrap();

        let mut fields = Map::new();
        fields.insert("status".into(), Value::String("completed".into()));
        update_job("job-u", fields).unwrap();

        let found = find_job("job-u").unwrap().unwrap();
        assert_eq!(found.status, "completed");
    });
}

#[test]
fn update_job_extra_fields() {
    run_in_temp(|| {
        let payload = json!({});
        add_job("job-x", "https://example.com", &payload, None).unwrap();

        let mut fields = Map::new();
        fields.insert("page_count".into(), Value::Number(42.into()));
        update_job("job-x", fields).unwrap();

        let found = find_job("job-x").unwrap().unwrap();
        assert_eq!(found.extra.get("page_count").unwrap(), &Value::Number(42.into()));
    });
}

#[test]
fn delete_jobs_removes_selected() {
    run_in_temp(|| {
        let payload = json!({});
        add_job("job-d1", "https://a.com", &payload, None).unwrap();
        add_job("job-d2", "https://b.com", &payload, None).unwrap();
        add_job("job-d3", "https://c.com", &payload, None).unwrap();

        delete_jobs(&["job-d1", "job-d3"]).unwrap();

        let jobs = load_jobs().unwrap();
        assert_eq!(jobs.len(), 1);
        assert_eq!(jobs[0].job_id, "job-d2");
    });
}

#[test]
fn get_jobs_by_status_filters() {
    run_in_temp(|| {
        let payload = json!({});
        add_job("j1", "https://a.com", &payload, None).unwrap();
        add_job("j2", "https://b.com", &payload, None).unwrap();

        let mut fields = Map::new();
        fields.insert("status".into(), Value::String("completed".into()));
        update_job("j1", fields).unwrap();

        let pending = get_jobs_by_status("pending").unwrap();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].job_id, "j2");

        let completed = get_jobs_by_status("completed").unwrap();
        assert_eq!(completed.len(), 1);
        assert_eq!(completed[0].job_id, "j1");
    });
}

#[test]
fn add_multiple_jobs_preserves_all() {
    run_in_temp(|| {
        let payload = json!({});
        add_job("m1", "https://a.com", &payload, None).unwrap();
        add_job("m2", "https://b.com", &payload, None).unwrap();
        add_job("m3", "https://c.com", &payload, None).unwrap();

        let jobs = load_jobs().unwrap();
        assert_eq!(jobs.len(), 3);
    });
}
