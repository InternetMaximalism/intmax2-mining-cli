#[derive(Debug, thiserror::Error)]
pub enum CLIError {
    #[error("Version error: {0}")]
    VersionError(String),
    #[error("IO error: {0}")]
    IoError(std::io::Error),
    #[error("Environment variable error: {0}")]
    EnvError(String),
    #[error("Internal error: {0}")]
    InternalError(String),
    #[error("Balance error: {0}")]
    BalanceError(String),
    #[error("Parse error: {0}")]
    ParseError(String),
    #[error("Network error: {0}")]
    NetworkError(String),
}
