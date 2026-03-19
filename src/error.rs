use serde_json::Value;

#[derive(Debug, thiserror::Error)]
pub enum BrowserflareError {
    #[error("CF_ACCOUNT_ID and CF_API_TOKEN must be set in .env")]
    MissingCredentials,

    #[error("HTTP {status}: {body}")]
    HttpError { status: u16, body: String },

    #[error("API error: {0}")]
    ApiError(Value),

    #[error("Crawl {status}: {result}")]
    CrawlFailed { status: String, result: Value },

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
}

pub type Result<T> = std::result::Result<T, BrowserflareError>;
