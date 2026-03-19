use std::collections::HashSet;
use std::path::Path;

use chrono::Local;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::config::JOBS_FILE;
use crate::error::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub job_id: String,
    pub url: String,
    pub label: Option<String>,
    pub payload: Value,
    pub started_at: String,
    pub status: String,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

pub fn load_jobs() -> Result<Vec<Job>> {
    let path = Path::new(JOBS_FILE);
    if !path.exists() {
        return Ok(Vec::new());
    }
    let content = std::fs::read_to_string(path)?;
    let jobs: Vec<Job> = serde_json::from_str(&content)?;
    Ok(jobs)
}

pub fn save_jobs(jobs: &[Job]) -> Result<()> {
    let json_str = serde_json::to_string_pretty(jobs)?;
    std::fs::write(JOBS_FILE, json_str)?;
    Ok(())
}

pub fn add_job(job_id: &str, url: &str, payload: &Value, label: Option<&str>) -> Result<Job> {
    let mut jobs = load_jobs()?;
    let job = Job {
        job_id: job_id.to_string(),
        url: url.to_string(),
        label: label.map(|s| s.to_string()),
        payload: payload.clone(),
        started_at: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        status: "pending".to_string(),
        extra: Map::new(),
    };
    jobs.push(job.clone());
    save_jobs(&jobs)?;
    Ok(job)
}

pub fn update_job(job_id: &str, fields: Map<String, Value>) -> Result<()> {
    let mut jobs = load_jobs()?;
    for job in &mut jobs {
        if job.job_id == job_id {
            for (key, value) in &fields {
                match key.as_str() {
                    "status" => {
                        if let Some(s) = value.as_str() {
                            job.status = s.to_string();
                        }
                    }
                    "label" => {
                        job.label = value.as_str().map(|s| s.to_string());
                    }
                    _ => {
                        job.extra.insert(key.clone(), value.clone());
                    }
                }
            }
            break;
        }
    }
    save_jobs(&jobs)?;
    Ok(())
}

pub fn find_job(job_id: &str) -> Result<Option<Job>> {
    let jobs = load_jobs()?;
    Ok(jobs.into_iter().find(|j| j.job_id == job_id))
}

pub fn delete_jobs(job_ids: &[&str]) -> Result<()> {
    let ids: HashSet<&str> = job_ids.iter().copied().collect();
    let jobs = load_jobs()?;
    let filtered: Vec<Job> = jobs.into_iter().filter(|j| !ids.contains(j.job_id.as_str())).collect();
    save_jobs(&filtered)?;
    Ok(())
}

pub fn get_jobs_by_status(status: &str) -> Result<Vec<Job>> {
    let jobs = load_jobs()?;
    Ok(jobs.into_iter().filter(|j| j.status == status).collect())
}
