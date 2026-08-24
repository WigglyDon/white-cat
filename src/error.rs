use std::fmt::{self, Display, Formatter};

pub type Result<T> = std::result::Result<T, WhiteCatError>;

#[derive(Debug)]
pub struct WhiteCatError {
    message: String,
}

impl WhiteCatError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl Display for WhiteCatError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for WhiteCatError {}

impl From<std::io::Error> for WhiteCatError {
    fn from(error: std::io::Error) -> Self {
        Self::new(error.to_string())
    }
}

impl From<image::ImageError> for WhiteCatError {
    fn from(error: image::ImageError) -> Self {
        Self::new(error.to_string())
    }
}

impl From<serde_json::Error> for WhiteCatError {
    fn from(error: serde_json::Error) -> Self {
        Self::new(error.to_string())
    }
}

pub fn fail<T>(message: impl Into<String>) -> Result<T> {
    Err(WhiteCatError::new(message))
}
