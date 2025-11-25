use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaybisData<T> {
    pub data: T,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum PaybisResponse<T> {
    Success(T),
    Error(PaybisError),
}

impl<T> From<PaybisResponse<T>> for Result<T, Box<dyn std::error::Error + Send + Sync>> {
    fn from(resp: PaybisResponse<T>) -> Self {
        match resp {
            PaybisResponse::Success(data) => Ok(data),
            PaybisResponse::Error(error) => Err(error.into_error()),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct PaybisError {
    pub message: String,
    pub code: String,
}

impl PaybisError {
    pub fn into_error(self) -> Box<dyn std::error::Error + Send + Sync> {
        format!("Paybis API error [{}]: {}", self.code, self.message).into()
    }
}
