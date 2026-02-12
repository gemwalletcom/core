use std::fmt::{self, Display, Formatter};

use serde_json::Value;

#[derive(Debug)]
pub enum RequestError {
    MissingParameter(String),
    InvalidFormat(String),
}

impl Display for RequestError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingParameter(name) => write!(f, "Missing {} parameter", name),
            Self::InvalidFormat(msg) => write!(f, "{}", msg),
        }
    }
}

impl From<RequestError> for String {
    fn from(error: RequestError) -> Self {
        error.to_string()
    }
}

pub trait ValueExt {
    fn get_param(&self, key: &str) -> Result<&Value, String>;
    fn get_str(&self, key: &str) -> Result<&str, String>;
}

impl ValueExt for Value {
    fn get_param(&self, key: &str) -> Result<&Value, String> {
        self.get(key).ok_or_else(|| RequestError::MissingParameter(key.into()).into())
    }

    fn get_str(&self, key: &str) -> Result<&str, String> {
        self.get(key).and_then(|v| v.as_str()).ok_or_else(|| RequestError::MissingParameter(key.into()).into())
    }
}
