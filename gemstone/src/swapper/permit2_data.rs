use crate::swapper::SwapperError;
use alloy_core::primitives::{Bytes, U256};
use alloy_primitives::aliases::{U160, U48};
use serde::{Deserialize, Serialize, Serializer};
use std::{
    fmt::{Debug, Display},
    str::FromStr,
};

use gem_evm::{
    permit2::{IAllowanceTransfer, Permit2Types},
    uniswap::command::Permit2Permit,
};
use primitives::{eip712::EIP712Domain, Chain};

#[derive(Debug, Clone, Serialize, Deserialize, uniffi::Record)]
pub struct Permit2Detail {
    pub token: String,
    pub amount: String,
    #[serde(serialize_with = "serialize_as_string")]
    pub expiration: u64,
    #[serde(serialize_with = "serialize_as_string")]
    pub nonce: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, uniffi::Record)]
pub struct PermitSingle {
    pub details: Permit2Detail,
    pub spender: String,
    #[serde(rename = "sigDeadline", serialize_with = "serialize_as_string")]
    pub sig_deadline: u64,
}

impl From<PermitSingle> for IAllowanceTransfer::PermitSingle {
    fn from(val: PermitSingle) -> Self {
        IAllowanceTransfer::PermitSingle {
            details: IAllowanceTransfer::PermitDetails {
                token: val.details.token.as_str().parse().unwrap(),
                amount: U160::from_str(&val.details.amount).unwrap(),
                expiration: U48::from(val.details.expiration),
                nonce: U48::from(val.details.nonce),
            },
            spender: val.spender.as_str().parse().unwrap(),
            sigDeadline: U256::from(val.sig_deadline),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, uniffi::Record)]
pub struct Permit2Data {
    pub permit_single: PermitSingle,
    pub signature: Vec<u8>,
}

impl From<Permit2Data> for Permit2Permit {
    fn from(val: Permit2Data) -> Self {
        Permit2Permit {
            permit_single: val.permit_single.into(),
            signature: Bytes::from(val.signature),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permit2Message {
    domain: EIP712Domain,
    types: Permit2Types,
    #[serde(rename = "primaryType")]
    primary_type: String,
    message: PermitSingle,
}

fn serialize_as_string<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
where
    T: Display,
    S: Serializer,
{
    serializer.serialize_str(&value.to_string())
}

#[uniffi::export]
pub fn permit2_data_to_eip712_json(chain: Chain, data: PermitSingle, contract: &str) -> Result<String, SwapperError> {
    let chain_id = chain.network_id();
    let message = Permit2Message {
        domain: EIP712Domain {
            name: "Permit2".to_string(),
            version: "".into(),
            chain_id: chain_id.parse::<u32>().unwrap(),
            verifying_contract: contract.to_string(),
        },
        types: Permit2Types::default(),
        primary_type: "PermitSingle".into(),
        message: data,
    };
    let json = serde_json::to_string(&message).map_err(|_| SwapperError::ABIError {
        msg: "failed to serialize EIP712 message to JSON".into(),
    })?;
    Ok(json)
}

#[cfg(test)]
mod tests {
    use gem_evm::uniswap::deployment::get_uniswap_permit2_by_chain;

    use super::*;
    #[test]
    fn test_permit2_data_eip712_json() {
        let data = PermitSingle {
            details: Permit2Detail {
                token: "0xdAC17F958D2ee523a2206206994597C13D831ec7".into(),
                amount: "1461501637330902918203684832716283019655932542975".into(),
                expiration: 1732780554,
                nonce: 0,
            },
            spender: "0x3fC91A3afd70395Cd496C647d5a6CC9D4B2b7FAD".into(),
            sig_deadline: 1730190354,
        };

        let json = permit2_data_to_eip712_json(Chain::Ethereum, data, get_uniswap_permit2_by_chain(&Chain::Ethereum).unwrap()).unwrap();
        assert_eq!(
            json,
            r#"{"domain":{"name":"Permit2","chainId":1,"verifyingContract":"0x000000000022D473030F116dDEE9F6B43aC78BA3"},"types":{"EIP712Domain":[{"name":"name","type":"string"},{"name":"chainId","type":"uint256"},{"name":"verifyingContract","type":"address"}],"PermitSingle":[{"name":"details","type":"PermitDetails"},{"name":"spender","type":"address"},{"name":"sigDeadline","type":"uint256"}],"PermitDetails":[{"name":"token","type":"address"},{"name":"amount","type":"uint160"},{"name":"expiration","type":"uint48"},{"name":"nonce","type":"uint48"}]},"primaryType":"PermitSingle","message":{"details":{"token":"0xdAC17F958D2ee523a2206206994597C13D831ec7","amount":"1461501637330902918203684832716283019655932542975","expiration":"1732780554","nonce":"0"},"spender":"0x3fC91A3afd70395Cd496C647d5a6CC9D4B2b7FAD","sigDeadline":"1730190354"}}"#
        );
    }
}
