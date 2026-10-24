use thiserror::Error;

#[derive(Debug, Error)]
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

pub type Result<T> = std::result::Result<T, ApiError>;
