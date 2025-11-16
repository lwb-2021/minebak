use std::backtrace::Backtrace;

use thiserror::Error;

pub type Result<T> = std::result::Result<T, MyError>;

#[derive(Debug, Error)]
pub enum MyError {
    #[error("{0}\n{1}")]
    IOError(#[from] std::io::Error, Backtrace),
    #[error("{0}\n{1}")]
    SerdeJsonError(#[from] serde_json::Error, Backtrace),
    #[error("{0}\n{1}")]
    WalkDirError(#[from] walkdir::Error, Backtrace),
    #[error("{0}")]
    Other(String),
}
