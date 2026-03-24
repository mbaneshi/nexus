//! Error types for the Nexus workspace.

/// All errors in the nexus workspace.
#[derive(Debug, thiserror::Error)]
pub enum NexusError {
    /// Storage / database errors.
    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),

    /// Filesystem IO errors.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    /// Scan / discovery errors.
    #[error("scan error: {0}")]
    Scan(String),

    /// Config management errors.
    #[error("config error: {0}")]
    Config(String),

    /// LLM / AI errors.
    #[error("ai error: {0}")]
    Ai(String),

    /// Serialization errors.
    #[error("serialization error: {0}")]
    Serialization(String),

    /// Generic internal error.
    #[error("{0}")]
    Internal(String),
}

/// Convenience result type for the nexus workspace.
pub type Result<T> = std::result::Result<T, NexusError>;
