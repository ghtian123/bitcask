use std::net::AddrParseError;
use std::string::FromUtf8Error;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum KvError {
    // #[error("{0}")]
    // AddrParseError(#[from] AddrParseError),
    #[error("{0}")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    Serde(#[from] serde_json::Error),

    #[error("Key not found")]
    KeyNotFound,

    #[error("sled error: {0}")]
    Sled(#[from] sled::Error),

    #[error("UTF-8 error: {0}")]
    Utf8(#[from] FromUtf8Error),

    #[error("{0}")]
    StringError(String),

    #[error("Unexpected command type")]
    UnexpectedCommandType,
}
