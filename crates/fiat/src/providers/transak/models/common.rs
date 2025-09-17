use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Response<T> {
    pub response: T,
}

#[derive(Debug, Deserialize)]
pub struct Data<T> {
    pub data: T,
}
