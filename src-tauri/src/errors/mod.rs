use serde::{Deserialize, Serialize};

pub type Result<T> = std::result::Result<T, MyError>;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MyError {
    code: ErrorCode,
    message: String,
}
impl std::fmt::Display for MyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("Error {:?}: {}", self.code, self.message))
    }
}

impl From<std::io::Error> for MyError {
    fn from(value: std::io::Error) -> Self {
        MyError {
            code: ErrorCode::IOError,
            message: format!("{:?}", value),
        }
    }
}

impl<T> From<std::sync::PoisonError<T>> for MyError {
    fn from(value: std::sync::PoisonError<T>) -> Self {
        MyError {
            code: ErrorCode::ConcurrentModification,
            message: format!("{:?}", value),
        }
    }
}

pub fn err<T>(code: ErrorCode, message: &str) -> Result<T> {
    Err(MyError {
        code,
        message: message.to_string(),
    })
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(untagged)]
#[repr(u8)]
pub enum ErrorCode {
    IOError = 0,
    ConcurrentModification = 1,
    InvalidParameter = 255,
}
