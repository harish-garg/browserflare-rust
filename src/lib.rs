pub mod error;
pub mod config;
pub mod payloads;
pub mod api;
pub mod screenshot_api;
pub mod pdf_api;
pub mod output;
pub mod screenshot_output;
pub mod pdf_output;
pub mod jobs;
pub mod batch;

// Re-export primary types and functions for convenience.
pub use error::{BrowserflareError, Result};
pub use config::{
    ApiConfig, TERMINAL_STATUSES, SUCCESS_STATUSES, FAILURE_STATUSES,
    RESOURCE_TYPES, DEFAULT_REJECT_RESOURCES, OUTPUT_FORMATS,
    SCREENSHOT_FORMATS, PDF_PAGE_FORMATS, WAIT_UNTIL_OPTIONS,
};
pub use payloads::{
    CrawlPayload, ScreenshotPayload, ScreenshotOptions, PdfPayload,
    CrawlResult, CrawlRecord, ScreenshotResult, PdfResult,
    Viewport, WaitForSelector, GotoOptions, JsonOptions, PdfOptions, PdfMargin,
    CfApiResponse,
};
pub use api::{start_crawl, get_crawl_status, get_crawl_results_paginated, cancel_crawl, poll_until_complete};
pub use screenshot_api::take_screenshot;
pub use pdf_api::generate_pdf;
pub use output::{
    sanitize_filename, save_results, load_result, search_results,
    get_statistics, diff_crawls, SearchMatch, CrawlStatistics, CrawlDiff,
};
pub use screenshot_output::{save_screenshot, log_screenshot};
pub use pdf_output::{save_pdf, log_pdf};
pub use jobs::{Job, load_jobs, save_jobs, add_job, update_job, find_job, delete_jobs, get_jobs_by_status};
pub use batch::{BatchEvent, load_urls, run_batch};
