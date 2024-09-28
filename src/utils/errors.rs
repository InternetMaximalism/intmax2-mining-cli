use std::fmt;

#[derive(Debug)]
pub enum CLIError {
    VersionError(String),
    IoError(std::io::Error),
    EnvError(String),
    InternalError(String),
    BalanceError(String),
    ParseError(String),
    NetworkError(String),
}

impl fmt::Display for CLIError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CLIError::VersionError(msg) => write!(f, "Version error: {}", msg),
            CLIError::IoError(err) => write!(f, "IO error: {}", err),
            CLIError::EnvError(msg) => write!(f, "Environment variable error: {}", msg),
            CLIError::InternalError(msg) => write!(f, "Internal error: {}", msg),
            CLIError::BalanceError(msg) => write!(f, "Balance error: {}", msg),
            CLIError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            CLIError::NetworkError(msg) => write!(f, "Network error: {}", msg),
        }
    }
}

impl std::error::Error for CLIError {}
