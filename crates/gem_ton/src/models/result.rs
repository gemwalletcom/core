#[derive(serde::Deserialize, serde::Serialize)]
pub struct TonResult<T> {
    pub result: T,
}
