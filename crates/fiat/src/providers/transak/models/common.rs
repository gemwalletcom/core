use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Response<T> {
    pub response: T,
}

#[derive(Debug, Deserialize)]
pub struct Data<T> {
    pub data: T,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum TransakResponse<T> {
    Success(Response<T>),
    Error(TransakError),
}

#[derive(Debug, Deserialize)]
pub struct TransakError {
    pub error: TransakErrorDetail,
}

#[derive(Debug, Deserialize)]
pub struct TransakErrorDetail {
    pub message: String,
}

impl<T> From<TransakResponse<T>> for Result<T, Box<dyn std::error::Error + Send + Sync>> {
    fn from(resp: TransakResponse<T>) -> Self {
        match resp {
            TransakResponse::Success(data) => Ok(data.response),
            TransakResponse::Error(error) => Err(error.error.message.into()),
        }
    }
}
