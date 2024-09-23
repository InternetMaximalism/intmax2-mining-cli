use std::fmt;

#[derive(Debug)]
pub enum CLIError {
    IoError(std::io::Error),
    EnvError(String),
    BalanceError(String),
    ParseError(String),
    NetworkError(String),
}

impl fmt::Display for CLIError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CLIError::IoError(err) => write!(f, "IO error: {}", err),
            CLIError::EnvError(msg) => write!(f, "Environment variable error: {}", msg),
            CLIError::BalanceError(msg) => write!(f, "Balance error: {}", msg),
            CLIError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            CLIError::NetworkError(msg) => write!(f, "Network error: {}", msg),
        }
    }
}

impl std::error::Error for CLIError {}
