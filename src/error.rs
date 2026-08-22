use std::error::Error;
use std::fmt::{Display, Formatter};

pub type Result<T> = std::result::Result<T, WhiteCatError>;

#[derive(Debug)]
pub struct WhiteCatError(pub String);

impl WhiteCatError {
    pub fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl Display for WhiteCatError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl Error for WhiteCatError {}

impl From<std::io::Error> for WhiteCatError {
    fn from(error: std::io::Error) -> Self {
        Self(error.to_string())
    }
}

impl From<serde_json::Error> for WhiteCatError {
    fn from(error: serde_json::Error) -> Self {
        Self(error.to_string())
    }
}

impl From<image::ImageError> for WhiteCatError {
    fn from(error: image::ImageError) -> Self {
        Self(error.to_string())
    }
}
