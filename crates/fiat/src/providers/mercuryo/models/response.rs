use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Response<T> {
    pub data: T,
}
