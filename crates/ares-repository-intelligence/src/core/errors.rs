use thiserror::Error;

#[derive(Error, Debug)]
pub enum EngineError {
    #[error("Database error: {0}")]
    DatabaseError(String),
    #[error("Execution failed: {0}")]
    ExecutionError(String),
    #[error("Capability not found: {0}")]
    CapabilityError(String),
    #[error("Internal error: {0}")]
    InternalError(String),
    #[error("Timeout: {0}")]
    Timeout(String),
}

pub type EngineResult<T> = Result<T, EngineError>;
