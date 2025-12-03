use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Data<T> {
    pub data: T,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum MoonPayResponse<T> {
    Success(T),
    Error(MoonPayError),
}

#[derive(Debug, Deserialize)]
pub struct MoonPayError {
    pub message: String,
}

impl<T> From<MoonPayResponse<T>> for Result<T, Box<dyn std::error::Error + Send + Sync>> {
    fn from(resp: MoonPayResponse<T>) -> Self {
        match resp {
            MoonPayResponse::Success(data) => Ok(data),
            MoonPayResponse::Error(error) => Err(error.message.into()),
        }
    }
}
