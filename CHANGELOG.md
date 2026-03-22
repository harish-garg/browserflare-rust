# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2026-03-22

### Breaking Changes

- `start_crawl`, `get_crawl_status`, `get_crawl_results_paginated`, `cancel_crawl`, and `poll_until_complete` now require an `&ApiConfig` parameter instead of calling `get_api_config()` internally.
- `take_screenshot` now requires an `&ApiConfig` parameter instead of calling `get_screenshot_api_config()` internally.
- `generate_pdf` now requires an `&ApiConfig` parameter instead of calling `get_pdf_api_config()` internally.
- `run_batch` now requires an `&ApiConfig` parameter.

### Added

- `test_config(base_url)` helper for building an `ApiConfig` that points at any URL, useful for testing against mock servers.
- Re-exported `get_api_config`, `get_screenshot_api_config`, `get_pdf_api_config`, and `test_config` from the crate root.
- `Debug` derive on `ScreenshotResult` and `PdfResult`.
- Comprehensive test suite (41 new integration tests, 49 total) covering:
  - Crawl API: start, status, cancel, poll, and paginated results with wiremock.
  - Screenshot API: success responses, JSON error responses, and HTTP errors.
  - PDF API: success responses, JSON error responses, HTTP errors, and HTML-to-PDF.
  - File I/O: save/load results, format-specific output, search, statistics, and diff.
  - Job management: add, find, update, delete, and status filtering.
- `wiremock` and `tempfile` as dev-dependencies.

### Migration Guide

All API functions now take `&ApiConfig` as an explicit parameter. Update call sites from:

```rust
let job_id = start_crawl(&client, &payload).await?;
```

to:

```rust
let config = get_api_config()?;
let job_id = start_crawl(&client, &config, &payload).await?;
```

The same pattern applies to `get_crawl_status`, `get_crawl_results_paginated`, `cancel_crawl`, `poll_until_complete`, `take_screenshot`, `generate_pdf`, and `run_batch`.

## [0.1.1] - 2025-01-01

### Changed

- Updated metadata and README for crates.io publish.
- Uncommented repository URL in Cargo.toml.

## [0.1.0] - 2025-01-01

### Added

- Initial release.
- Crawl API: start, status, paginated results, cancel, and poll-until-complete.
- Screenshot API with PNG, JPEG, and WebP support.
- PDF generation from URL or HTML.
- Output management: save, load, search, statistics, and diff.
- Job tracking with JSON persistence.
- Batch crawling with event callbacks.
