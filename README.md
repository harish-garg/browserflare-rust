# browserflare

Rust client for [Cloudflare Browser Rendering](https://developers.cloudflare.com/browser-rendering/) APIs.

Supports crawling, screenshots, and PDF generation.

> Prefer a GUI App? Look at our desktop app - [browserflare.xyz](https://browserflare.xyz/).

## Installation

```toml
[dependencies]
browserflare = "0.1.1"
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
use browserflare::{ScreenshotPayload, take_screenshot, save_screenshot};

#[tokio::main]
async fn main() -> browserflare::Result<()> {
    let client = reqwest::Client::new();
    let payload = ScreenshotPayload {
        url: "https://browserflare.xyz".into(),
        ..Default::default()
    };
    let result = take_screenshot(&client, &payload).await?;
    save_screenshot("https://browserflare.xyz", &result.bytes, &result.content_type, None)?;
    Ok(())
}
```

### Crawl

```rust
use browserflare::{CrawlPayload, start_crawl, poll_until_complete, save_results};

#[tokio::main]
async fn main() -> browserflare::Result<()> {
    let client = reqwest::Client::new();
    let payload = CrawlPayload {
        url: "https://browserflare.xyz".into(),
        limit: Some(10),
        formats: Some(vec!["markdown".into()]),
        ..Default::default()
    };
    let job_id = start_crawl(&client, &payload).await?;
    let result = poll_until_complete(&client, &job_id, 3, None).await?;
    save_results(&job_id, &result, Some(&["markdown".into()]))?;
    Ok(())
}
```

### PDF

```rust
use browserflare::{PdfPayload, generate_pdf, save_pdf};

#[tokio::main]
async fn main() -> browserflare::Result<()> {
    let client = reqwest::Client::new();
    let payload = PdfPayload {
        url: Some("https://browserflare.xyz".into()),
        ..Default::default()
    };
    let result = generate_pdf(&client, &payload).await?;
    save_pdf("https://browserflare.xyz", &result.bytes, None)?;
    Ok(())
}
```

## License

MIT
