use std::fmt;

#[derive(Debug)]
pub enum CLIError {
    IoError(std::io::Error),
    ParseError(String),
    NetworkError(String),
}

impl fmt::Display for CLIError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CLIError::IoError(err) => write!(f, "IO error: {}", err),
            CLIError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            CLIError::NetworkError(msg) => write!(f, "Network error: {}", msg),
        }
    }
}

impl std::error::Error for CLIError {}
