use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResult<T> {
    pub ok: bool,
    pub result: T,
}

