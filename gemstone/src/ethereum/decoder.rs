use crate::GemstoneError;
use gem_evm::call_decoder;

pub type GemDecodedCall = call_decoder::DecodedCall;
pub type GemDecodedCallParam = call_decoder::DecodedCallParam;

#[uniffi::remote(Record)]
pub struct GemDecodedCall {
    pub function: String,
    pub params: Vec<GemDecodedCallParam>,
}

#[uniffi::remote(Record)]
pub struct GemDecodedCallParam {
    pub name: String,
    pub r#type: String,
    pub value: String,
}

#[derive(Debug, Default, uniffi::Object)]
pub struct EthereumDecoder;

impl EthereumDecoder {
    pub fn decode_call_internal(calldata: &str, abi: Option<&str>) -> Result<GemDecodedCall, Box<dyn std::error::Error + Send + Sync>> {
        call_decoder::decode_call(calldata, abi)
    }
}

#[uniffi::export]
impl EthereumDecoder {
    #[uniffi::constructor]
    pub fn new() -> Self {
        Self {}
    }

    pub fn decode_call(&self, calldata: String, abi: Option<String>) -> Result<GemDecodedCall, GemstoneError> {
        Self::decode_call_internal(&calldata, abi.as_deref()).map_err(GemstoneError::from)
    }
}
