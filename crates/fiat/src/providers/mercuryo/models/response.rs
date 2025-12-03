use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Response<T> {
    pub data: T,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum MercuryoResponse<T> {
    Success(Response<T>),
    Error(MercuryoError),
}

#[derive(Debug, Deserialize)]
pub struct MercuryoError {
    pub message: String,
}

impl<T> From<MercuryoResponse<T>> for Result<T, Box<dyn std::error::Error + Send + Sync>> {
    fn from(resp: MercuryoResponse<T>) -> Self {
        match resp {
            MercuryoResponse::Success(data) => Ok(data.data),
            MercuryoResponse::Error(error) => Err(error.message.into()),
        }
    }
}
