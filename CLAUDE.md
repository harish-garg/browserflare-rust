# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

Rust client library for [Cloudflare Browser Rendering](https://developers.cloudflare.com/browser-rendering/) APIs (crawl, screenshot, PDF, content). Published to crates.io as `browserflare`.

## Build & Test Commands

```powershell
cargo build                        # Build the library
cargo test                         # Run all 53 tests
cargo test --test crawl_api        # Run a single test file
cargo test test_name               # Run a specific test by name
cargo test -- --nocapture          # Show println output during tests
cargo clippy                       # Lint
cargo doc --open                   # Generate and view docs
```

## Architecture

### Module Organization

The crate is organized into three layers:

1. **API layer** — HTTP calls to Cloudflare endpoints
   - `api.rs` — Crawl API: `start_crawl`, `get_crawl_status`, `get_crawl_results_paginated`, `cancel_crawl`, `poll_until_complete`
   - `screenshot_api.rs` — `take_screenshot`
   - `pdf_api.rs` — `generate_pdf`
   - `content_api.rs` — `fetch_content`

2. **Persistence layer** — File I/O and job tracking
   - `output.rs` — Save/load crawl results, search, statistics, diff (writes to `output/{job_id}/`)
   - `screenshot_output.rs` — Save screenshots + append to `screenshot_log.json`
   - `pdf_output.rs` — Save PDFs + append to `pdf_log.json`
   - `jobs.rs` — CRUD for job records in `crawl_jobs.json`

3. **Orchestration layer** — Higher-level workflows
   - `batch.rs` — Batch URL processing with `BatchEvent` callbacks

Supporting modules:
- `config.rs` — `ApiConfig` struct, credential loading from env vars (`CF_ACCOUNT_ID`, `CF_API_TOKEN`), constants
- `payloads.rs` — All request/response types with serde derives (`camelCase`, `skip_serializing_if`)
- `error.rs` — `BrowserflareError` enum, `Result<T>` alias
- `lib.rs` — Re-exports everything for flat `use browserflare::*` access

### Key Design Pattern: Explicit Config

All API functions take `&ApiConfig` as an explicit parameter (not fetched internally). This was a v0.2.0 breaking change for testability. Production code calls `get_api_config()` / `get_screenshot_api_config()` / `get_pdf_api_config()` / `get_content_api_config()` to build configs from env vars. Tests use `test_config(base_url)` to point at wiremock mock servers.

All API functions also take `&reqwest::Client` by reference — the caller owns the client.

### Callback Conventions

- `poll_until_complete` takes `Option<&dyn Fn(&str, &CrawlResult)>` for status updates
- `get_crawl_results_paginated` takes `Option<&dyn Fn(usize, usize)>` for progress
- `run_batch` takes `&dyn Fn(BatchEvent)` for event notifications
- Callbacks use `Fn` (not `FnMut`). In tests, use `RefCell` to capture state inside `Fn` closures.

## Testing Patterns

### Mock HTTP tests (`tests/crawl_api.rs`, `screenshot_api.rs`, `pdf_api.rs`, `content_api.rs`)

Use `wiremock::MockServer` to stand up a local HTTP server, mount response mocks, and pass `test_config(&server.uri())` to API functions.

### File I/O tests (`tests/output_tests.rs`, `jobs_tests.rs`)

Use `tempfile::TempDir` + `std::env::set_current_dir` for isolated filesystem. Because `set_current_dir` is process-global, all such tests **must** serialize through `static CWD_LOCK: Mutex<()>`. The `run_in_temp` helper handles lock acquisition and CWD save/restore.

### Test data helpers

`output_tests.rs` has `make_result()` and `make_record()` helpers for constructing test `CrawlResult`/`CrawlRecord` instances.

## Serde Conventions in `payloads.rs`

- All payload structs use `#[serde(rename_all = "camelCase")]` to match Cloudflare's JSON API
- Optional fields use `#[serde(skip_serializing_if = "Option::is_none")]` for clean requests
- `CrawlPayload`, `CrawlResult`, `CrawlRecord`, and `Job` use `#[serde(flatten)]` with `extra: Map<String, Value>` for forward-compatible extensibility
- Custom `deserialize_cursor` handles Cloudflare returning cursor as either string or number
