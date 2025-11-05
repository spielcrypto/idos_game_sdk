/// Error types for iDos Games SDK
use thiserror::Error;

pub type IdosResult<T> = Result<T, IdosError>;

#[derive(Error, Debug)]
pub enum IdosError {
    #[error("Authentication error: {0}")]
    Auth(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("API error: {0}")]
    Api(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Invalid configuration: {0}")]
    Config(String),

    #[error("Payment error: {0}")]
    Payment(String),

    #[error("Crypto wallet error: {0}")]
    Wallet(String),

    #[error("Not supported on this platform: {0}")]
    PlatformNotSupported(String),

    #[error("Unknown error: {0}")]
    Unknown(String),

    // Additional error types for better granularity
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    #[error("Timeout error: {0}")]
    TimeoutError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),
}
