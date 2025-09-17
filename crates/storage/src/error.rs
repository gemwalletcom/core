use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub enum DatabaseError {
    NotFound,
    Internal(String),
}

impl fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DatabaseError::NotFound => write!(f, "Resource not found"),
            DatabaseError::Internal(msg) => write!(f, "Internal database error: {}", msg),
        }
    }
}

impl Error for DatabaseError {}

impl From<diesel::result::Error> for DatabaseError {
    fn from(error: diesel::result::Error) -> Self {
        match error {
            diesel::result::Error::NotFound => DatabaseError::NotFound,
            e => DatabaseError::Internal(e.to_string()),
        }
    }
}
