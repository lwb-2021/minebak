use std::backtrace::Backtrace;

use serde::Serialize;
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

impl Serialize for MyError {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let err = self.to_string();
        log::error!("{}", err);
        log::error!("Sending error to frontend");
        serializer.serialize_str(&err)
    }
}
