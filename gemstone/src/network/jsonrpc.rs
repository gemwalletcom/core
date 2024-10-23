#[derive(Debug, Clone, uniffi::Record)]
pub struct JsonRpcRequest {
    pub method: String,
    pub params: Option<String>, // json string
    pub id: u64,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct JsonRpcResponse {
    pub result: Option<String>,
    pub error: Option<JsonRpcError>,
    pub id: u64,
}

#[derive(Debug, Clone, uniffi::Enum)]
pub enum JsonRpcResult {
    Value(JsonRpcResponse),
    Error(JsonRpcError),
}
