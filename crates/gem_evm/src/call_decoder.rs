use alloy_dyn_abi::{DynSolValue, JsonAbiExt};
use alloy_json_abi::JsonAbi;
use alloy_primitives::hex;
use alloy_sol_types::SolInterface;
use anyhow::{anyhow, Result};

use crate::erc20::IERC20::IERC20Calls;

#[derive(Debug, PartialEq)]
pub struct DecodedCallParam {
    pub name: String,
    pub r#type: String,
    pub value: String,
}

#[derive(Debug, PartialEq)]
pub struct DecodedCall {
    pub function: String,
    pub params: Vec<DecodedCallParam>,
}

pub fn decode_call(calldata: &str, abi: Option<&str>) -> Result<DecodedCall> {
    let calldata = hex::decode(calldata)?;

    // Check minimum calldata length early
    if calldata.len() < 4 {
        return Err(anyhow!("Calldata too short"));
    }

    // Try ERC20 interface first if no ABI provided
    if abi.is_none() {
        if let Ok(call) = IERC20Calls::abi_decode(&calldata) {
            return Ok(call.into());
        }
    }

    if let Some(abi_str) = abi {
        let abi = serde_json::from_str::<JsonAbi>(abi_str)?;
        let selector = &calldata[..4];

        for function in abi.functions() {
            if function.selector() == selector {
                if let Ok(params) = function.abi_decode_input(&calldata[4..]) {
                    return Ok(DecodedCall {
                        function: function.name.clone(),
                        params: function
                            .inputs
                            .iter()
                            .zip(params.iter())
                            .map(|(input, output)| DecodedCallParam {
                                name: input.name.clone(),
                                r#type: input.ty.to_string(),
                                value: format_param_value(output),
                            })
                            .collect(),
                    });
                } else {
                    return Err(anyhow!("Failed to decode function parameters for {}", function.name));
                }
            }
        }
        return Err(anyhow!("No matching function found for selector {:02x?}", selector));
    }

    Err(anyhow!("Failed to decode calldata"))
}

pub fn format_param_value(value: &DynSolValue) -> String {
    match value {
        DynSolValue::Address(addr) => addr.to_string(),
        DynSolValue::Uint(val, _) => val.to_string(),
        DynSolValue::Int(val, _) => val.to_string(),
        DynSolValue::Bool(val) => val.to_string(),
        DynSolValue::Bytes(val) => format!("0x{}", hex::encode(val)),
        DynSolValue::FixedBytes(val, _) => format!("0x{}", hex::encode(val)),
        DynSolValue::String(val) => val.clone(),
        DynSolValue::Array(vals) | DynSolValue::FixedArray(vals) => {
            let formatted: Vec<String> = vals.iter().map(format_param_value).collect();
            format!("[{}]", formatted.join(", "))
        }
        DynSolValue::Tuple(vals) => {
            let formatted: Vec<String> = vals.iter().map(format_param_value).collect();
            format!("({})", formatted.join(", "))
        }
        _ => format!("{value:?}"),
    }
}

impl From<IERC20Calls> for DecodedCall {
    fn from(call: IERC20Calls) -> Self {
        let (function, params) = match call {
            IERC20Calls::transfer(transfer) => (
                "transfer",
                vec![
                    ("to", "address", transfer.to.to_string()),
                    ("value", "uint256", transfer.value.to_string()),
                ]
            ),
            IERC20Calls::transferFrom(transfer_from) => (
                "transferFrom", 
                vec![
                    ("from", "address", transfer_from.from.to_string()),
                    ("to", "address", transfer_from.to.to_string()),
                    ("value", "uint256", transfer_from.value.to_string()),
                ]
            ),
            IERC20Calls::approve(approve) => (
                "approve",
                vec![
                    ("spender", "address", approve.spender.to_string()),
                    ("value", "uint256", approve.value.to_string()),
                ]
            ),
            IERC20Calls::name(_) => (
                "name",
                vec![]
            ),
            IERC20Calls::symbol(_) => (
                "symbol", 
                vec![]
            ),
            IERC20Calls::decimals(_) => (
                "decimals",
                vec![]
            ),
            IERC20Calls::allowance(allowance) => (
                "allowance",
                vec![
                    ("owner", "address", allowance.owner.to_string()),
                    ("spender", "address", allowance.spender.to_string()),
                ]
            ),
        };

        DecodedCall {
            function: function.to_string(),
            params: params.into_iter()
                .map(|(name, r#type, value)| DecodedCallParam {
                    name: name.to_string(),
                    r#type: r#type.to_string(),
                    value,
                })
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_erc20_transfer() {
        let calldata =
            "0xa9059cbb0000000000000000000000002df1c51e09aecf9cacb7bc98cb1742757f163df700000000000000000000000000000000000000000000000000000000005ec1d0";
        let decoded = decode_call(calldata, None).unwrap();

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
        let decoded = decode_call(calldata, Some(abi)).unwrap();

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
        let result = decode_call("0x1234", None);  // Only 2 bytes, need 4
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Calldata too short"));
    }

    #[test]
    fn test_decode_erc20_view_functions() {
        // Test ERC20 name() function - 0x06fdde03
        let name_calldata = "0x06fdde03";
        let name_result = decode_call(name_calldata, None).unwrap();
        assert_eq!(name_result.function, "name");
        assert_eq!(name_result.params.len(), 0);

        // Test ERC20 allowance(address,address) function - 0xdd62ed3e
        let allowance_calldata = "0xdd62ed3e0000000000000000000000008ba1f109551bd432803012645aac136c0c3def25000000000000000000000000271682deb8c4e0901d1a1550ad2e64d568e69909";
        let allowance_result = decode_call(allowance_calldata, None).unwrap();
        assert_eq!(allowance_result.function, "allowance");
        assert_eq!(allowance_result.params.len(), 2);
        assert_eq!(allowance_result.params[0].name, "owner");
        assert_eq!(allowance_result.params[0].r#type, "address");
        assert_eq!(allowance_result.params[0].value, "0x8Ba1f109551bd432803012645aAC136C0c3Def25");
        assert_eq!(allowance_result.params[1].name, "spender");
        assert_eq!(allowance_result.params[1].r#type, "address");
        assert_eq!(allowance_result.params[1].value, "0x271682DEB8C4E0901D1a1550aD2e64D568E69909");
    }
}