use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum ProviderError {
    #[error("Authentication failed")]
    Authentication,

    #[error("Rate limited by provider")]
    RateLimited,

    #[error("Request timed out")]
    Timeout,

    #[error("Connection failed")]
    ConnectionFailed,

    #[error("Invalid request payload")]
    InvalidRequest,

    #[error("Invalid or unexpected response from provider")]
    InvalidResponse,

    #[error("Provider is currently unavailable")]
    ProviderUnavailable,

    #[error("Budget exceeded for this operation")]
    BudgetExceeded,

    #[error("Circuit breaker is open")]
    CircuitOpen,

    #[error("Unknown error: {0}")]
    Unknown(String),
}
