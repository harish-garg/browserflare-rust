# browserflare

Rust client for [Cloudflare Browser Rendering](https://developers.cloudflare.com/browser-rendering/) APIs.

Supports crawling, screenshots, and PDF generation.

> Prefer a GUI App? Look at our desktop app - [browserflare.xyz](https://browserflare.xyz/).

## What's New in 0.2.0

All API functions (`start_crawl`, `take_screenshot`, `generate_pdf`, etc.) now take an explicit `&ApiConfig` parameter instead of fetching credentials internally. This makes the library testable against mock servers via the new `test_config()` helper. See [CHANGELOG.md](CHANGELOG.md) for the full list of changes and a migration guide.

## Installation

```toml
[dependencies]
browserflare = "0.2.0"
tokio = { version = "1", features = ["full"] }
reqwest = "0.12"
```

## Setup

Set your Cloudflare credentials as environment variables (or in a `.env` file):

```
CF_ACCOUNT_ID=your_account_id
CF_API_TOKEN=your_api_token
```

## Usage

### Screenshot

```rust
use browserflare::{
    get_screenshot_api_config, take_screenshot, save_screenshot, ScreenshotPayload,
};

#[tokio::main]
async fn main() -> browserflare::Result<()> {
    let client = reqwest::Client::new();
    let config = get_screenshot_api_config()?;
    let payload = ScreenshotPayload {
        url: "https://browserflare.xyz".into(),
        ..Default::default()
    };
    let result = take_screenshot(&client, &config, &payload).await?;
    save_screenshot("https://browserflare.xyz", &result.bytes, &result.content_type, None)?;
    Ok(())
}
```

### Crawl

```rust
use browserflare::{
    get_api_config, start_crawl, poll_until_complete, save_results, CrawlPayload,
};

#[tokio::main]
async fn main() -> browserflare::Result<()> {
    let client = reqwest::Client::new();
    let config = get_api_config()?;
    let payload = CrawlPayload {
        url: "https://browserflare.xyz".into(),
        limit: Some(10),
        formats: Some(vec!["markdown".into()]),
        ..Default::default()
    };
    let job_id = start_crawl(&client, &config, &payload).await?;
    let result = poll_until_complete(&client, &config, &job_id, 3, None).await?;
    save_results(&job_id, &result, Some(&["markdown".into()]))?;
    Ok(())
}
```

### PDF

```rust
use browserflare::{get_pdf_api_config, generate_pdf, save_pdf, PdfPayload};

#[tokio::main]
async fn main() -> browserflare::Result<()> {
    let client = reqwest::Client::new();
    let config = get_pdf_api_config()?;
    let payload = PdfPayload {
        url: Some("https://browserflare.xyz".into()),
        ..Default::default()
    };
    let result = generate_pdf(&client, &config, &payload).await?;
    save_pdf("https://browserflare.xyz", &result.bytes, None)?;
    Ok(())
}
```

## Testing

The library includes 49 tests covering API calls (via mock HTTP), file I/O, and job management.

```bash
cargo test
```

To test against a mock server in your own code, use the `test_config` helper:

```rust
use browserflare::{test_config, start_crawl, CrawlPayload};

let config = test_config("http://localhost:8080");
let result = start_crawl(&client, &config, &payload).await;
```

## License

MIT
