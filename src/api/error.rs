use thiserror::Error;
#[derive(Debug, Error)]
#[allow(dead_code)]
pub enum ApiError {
    #[error("network error: {0}")]
    Network(String),
    #[error("http {0}")]
    Http(u16),
    #[error("not found")]
    NotFound,
    #[error("rate limited")]
    RateLimited,
    #[error("cancelled")]
    Cancelled,
}

#[allow(dead_code)]
pub type Result<T> = std::result::Result<T, ApiError>;
