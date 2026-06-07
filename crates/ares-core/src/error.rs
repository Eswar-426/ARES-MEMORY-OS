use thiserror::Error;

/// Unified error type for all ARES operations.
///
/// Variants are kept coarse at the boundary level — callers distinguish
/// cases using the variant, not string matching on messages.
///
/// Note: Database and Pool variants carry String messages rather than
/// the concrete rusqlite/r2d2 types, so ares-core does not depend on
/// those crates. ares-store implements the From conversions locally.
#[derive(Debug, Error)]
pub enum AresError {
    // ----------------------------------------------------------------
    // Domain errors
    // ----------------------------------------------------------------
    #[error("Not found: {resource_type} with id '{id}'")]
    NotFound { resource_type: &'static str, id: String },

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Not initialized: {0}")]
    NotInitialized(String),

    #[error("Scan already in progress for project {project_id}")]
    ScanInProgress { project_id: String },

    #[error("Decision is already superseded: {id}")]
    AlreadySuperseded { id: String },

    // ----------------------------------------------------------------
    // Infrastructure errors
    // ----------------------------------------------------------------
    #[error("Database error: {0}")]
    Database(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Path is invalid or escapes project root: {0}")]
    InvalidPath(String),

    #[error("Migration failed: {0}")]
    Migration(String),

    // ----------------------------------------------------------------
    // IPC errors
    // ----------------------------------------------------------------
    #[error("IPC connection failed: {0}")]
    IpcConnection(String),

    #[error("IPC request timed out after {ms}ms")]
    IpcTimeout { ms: u64 },

    #[error("Unknown IPC method: {0}")]
    UnknownMethod(String),

    // ----------------------------------------------------------------
    // Scanner errors
    // ----------------------------------------------------------------
    #[error("Parse error in {file}: {message}")]
    ParseError { file: String, message: String },

    #[error("Unsupported language: {0}")]
    UnsupportedLanguage(String),
}

impl AresError {
    // ----------------------------------------------------------------
    // Constructors for common variants
    // ----------------------------------------------------------------

    pub fn not_found(resource_type: &'static str, id: impl Into<String>) -> Self {
        Self::NotFound { resource_type, id: id.into() }
    }

    pub fn conflict(msg: impl Into<String>) -> Self {
        Self::Conflict(msg.into())
    }

    pub fn validation(msg: impl Into<String>) -> Self {
        Self::Validation(msg.into())
    }

    pub fn not_initialized(msg: impl Into<String>) -> Self {
        Self::NotInitialized(msg.into())
    }

    pub fn invalid_path(msg: impl Into<String>) -> Self {
        Self::InvalidPath(msg.into())
    }

    pub fn migration(msg: impl Into<String>) -> Self {
        Self::Migration(msg.into())
    }

    pub fn ipc_connection(msg: impl Into<String>) -> Self {
        Self::IpcConnection(msg.into())
    }

    pub fn db(msg: impl ToString) -> Self {
        Self::Database(msg.to_string())
    }

    // ----------------------------------------------------------------
    // IPC error code mapping (used by IPC protocol layer)
    // ----------------------------------------------------------------

    pub fn ipc_code(&self) -> i32 {
        match self {
            Self::NotFound { .. }          => 404,
            Self::Conflict(_)              => 409,
            Self::Validation(_)            => 400,
            Self::NotInitialized(_)        => 1001,
            Self::ScanInProgress { .. }    => 1002,
            Self::AlreadySuperseded { .. } => 409,
            Self::UnknownMethod(_)         => 400,
            Self::InvalidPath(_)           => 400,
            Self::IpcConnection(_)         => 503,
            Self::IpcTimeout { .. }        => 504,
            Self::UnsupportedLanguage(_)   => 400,
            _                              => 500,
        }
    }
}

impl From<serde_json::Error> for AresError {
    fn from(e: serde_json::Error) -> Self {
        Self::Serialization(e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn not_found_has_correct_code() {
        let e = AresError::not_found("memory", "mem_123");
        assert_eq!(e.ipc_code(), 404);
        assert!(e.to_string().contains("mem_123"));
    }

    #[test]
    fn validation_has_correct_code() {
        let e = AresError::validation("title too long");
        assert_eq!(e.ipc_code(), 400);
    }

    #[test]
    fn scan_in_progress_has_correct_code() {
        let e = AresError::ScanInProgress { project_id: "proj_1".into() };
        assert_eq!(e.ipc_code(), 1002);
    }

    #[test]
    fn database_error_code_is_500() {
        let e = AresError::db("connection refused");
        assert_eq!(e.ipc_code(), 500);
    }

    #[test]
    fn other_error_codes() {
        let e_conflict = AresError::conflict("already exists");
        assert_eq!(e_conflict.ipc_code(), 409);
        assert!(e_conflict.to_string().contains("already exists"));

        let e_init = AresError::not_initialized("missing store");
        assert_eq!(e_init.ipc_code(), 1001);

        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied");
        let e_io = AresError::from(io_err);
        assert_eq!(e_io.ipc_code(), 500);
        assert!(e_io.to_string().contains("denied"));
    }
}
