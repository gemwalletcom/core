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
    pub fn decode_call_internal(calldata: &str, abi: Option<&str>) -> anyhow::Result<GemDecodedCall> {
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


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_erc20_transfer() {
        let calldata =
            "0xa9059cbb0000000000000000000000002df1c51e09aecf9cacb7bc98cb1742757f163df700000000000000000000000000000000000000000000000000000000005ec1d0";
        let decoded = EthereumDecoder::decode_call_internal(calldata, None).unwrap();

        assert_eq!(decoded.function, "transfer");
        assert_eq!(decoded.params[0].name, "to");
        assert_eq!(decoded.params[0].r#type, "address");
        assert_eq!(decoded.params[0].value, "0x2Df1c51E09aECF9cacB7bc98cB1742757f163dF7");
        assert_eq!(decoded.params[1].name, "value");
        assert_eq!(decoded.params[1].r#type, "uint256");
        assert_eq!(decoded.params[1].value, "6210000");
    }

    #[test]
    fn test_decode_custom_abi() {
        // Using ERC721 safeTransferFrom as test case
        let calldata = "0x42842e0e0000000000000000000000008ba1f109551bd432803012645aac136c0c3def25000000000000000000000000271682deb8c4e0901d1a1550ad2e64d568e69909000000000000000000000000000000000000000000000000000000000000007b";
        let abi = r#"[
    {
        "inputs": [
            {
                "name": "from",
                "type": "address"
            },
            {
                "name": "to",
                "type": "address"
            },
            {
                "name": "tokenId",
                "type": "uint256"
            }
        ],
        "name": "safeTransferFrom",
        "outputs": [],
        "stateMutability": "nonpayable",
        "type": "function"
    }
]
"#;
        let decoded = EthereumDecoder::decode_call_internal(calldata, Some(abi)).unwrap();

        assert_eq!(decoded.function, "safeTransferFrom");
        assert_eq!(decoded.params.len(), 3);
        assert_eq!(decoded.params[0].name, "from");
        assert_eq!(decoded.params[0].r#type, "address");
        assert_eq!(decoded.params[0].value, "0x8Ba1f109551bd432803012645aAC136C0c3Def25");
        assert_eq!(decoded.params[1].name, "to");
        assert_eq!(decoded.params[1].r#type, "address");
        assert_eq!(decoded.params[1].value, "0x271682DEB8C4E0901D1a1550aD2e64D568E69909");
        assert_eq!(decoded.params[2].name, "tokenId");
        assert_eq!(decoded.params[2].r#type, "uint256");
        assert_eq!(decoded.params[2].value, "123");
    }

    #[test]
    fn test_decode_short_calldata() {
        // Test that short calldata returns proper error
        let result = EthereumDecoder::decode_call_internal("0x1234", None); // Only 2 bytes, need 4
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Calldata too short"));
    }
}
