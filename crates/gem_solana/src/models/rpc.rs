use serde::{Deserialize, Serialize};

pub const ENCODING_BASE64: &str = "base64";
pub const ENCODING_BASE58: &str = "base58";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Configuration {
    pub commitment: &'static str,
    pub encoding: &'static str,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub filters: Vec<Filter>,
}

impl Configuration {
    pub fn new(filters: Vec<Filter>) -> Self {
        Self {
            commitment: "confirmed",
            encoding: "base64",
            filters,
        }
    }
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            commitment: "confirmed",
            encoding: "base64",
            filters: vec![],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Filter {
    pub memcmp: Memcmp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memcmp {
    pub offset: u8,
    pub bytes: String,
    pub encoding: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValueResult<T> {
    pub value: T,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValueData<T> {
    pub data: T,
    pub owner: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parsed<T> {
    pub parsed: T,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Info<T> {
    pub info: T,
}

pub type AccountData = ValueData<Vec<String>>;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    pub ok: Option<String>,
}
