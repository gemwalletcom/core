use alloy_core::primitives::{Address, Bytes, U256};
use alloy_primitives::aliases::{U160, U48};
use serde::{Deserialize, Serialize, Serializer};
use std::{
    fmt::{Debug, Display},
    str::FromStr,
};

use gem_evm::{
    permit2::{IAllowanceTransfer, Permit2Types},
    uniswap::{command::Permit2Permit, deployment::get_deployment_by_chain},
};
use primitives::{eip712::EIP712Domain, AssetId, Chain, ChainType};

static DEFAULT_SLIPPAGE_BPS: u32 = 300;

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum SwapperError {
    #[error("Not supported chain")]
    NotSupportedChain,
    #[error("Invalid address {address}")]
    InvalidAddress { address: String },
    #[error("Invalid amount")]
    InvalidAmount,
    #[error("RPC error: {msg}")]
    NetworkError { msg: String },
    #[error("ABI error: {msg}")]
    ABIError { msg: String },
    #[error("No quote available")]
    NoQuoteAvailable,
    #[error("Not implemented")]
    NotImplemented,
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemSwapMode {
    ExactIn,
    ExactOut,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct SwapQuoteRequest {
    pub from_asset: AssetId,
    pub to_asset: AssetId,
    pub wallet_address: String,
    pub destination_address: String,
    pub value: String,
    pub mode: GemSwapMode,
    pub options: Option<GemSwapOptions>,
}

#[derive(Debug, Default, Clone, uniffi::Record)]
pub struct GemSwapFee {
    pub bps: u32,
    pub address: String,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemSwapOptions {
    pub slippage_bps: u32,
    pub fee: Option<GemSwapFee>,
    pub preferred_providers: Vec<String>,
}

impl Default for GemSwapOptions {
    fn default() -> Self {
        Self {
            slippage_bps: DEFAULT_SLIPPAGE_BPS,
            fee: None,
            preferred_providers: vec![],
        }
    }
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct SwapQuote {
    pub chain_type: ChainType,
    pub from_value: String,
    pub to_value: String,
    pub provider: SwapProviderData,
    pub approval: ApprovalType,
    pub request: SwapQuoteRequest,
}

#[derive(Debug, Clone, uniffi::Enum)]
pub enum ApprovalType {
    Approve(ApprovalData),
    Permit2(ApprovalData),
    None,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct ApprovalData {
    pub token: String,
    pub spender: String,
    pub value: String,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct SwapQuoteData {
    pub to: String,
    pub value: String,
    pub data: String,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct SwapProviderData {
    pub name: String,
    pub routes: Vec<SwapRoute>,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct SwapRoute {
    pub route_type: String,
    pub input: String,
    pub output: String,
    pub fee_tier: String,
}

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
                token: Address::parse_checksummed(val.details.token, None).unwrap(),
                amount: U160::from_str(&val.details.amount).unwrap(),
                expiration: U48::from(val.details.expiration),
                nonce: U48::from(val.details.nonce),
            },
            spender: Address::parse_checksummed(val.spender, None).unwrap(),
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
pub fn permit2_data_to_eip712_json(chain: Chain, data: PermitSingle) -> String {
    let chain_id = chain.network_id();
    let contract = get_deployment_by_chain(chain).unwrap().permit2;
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
    serde_json::to_string(&message).unwrap()
}

#[cfg(test)]
mod tests {
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

        let json = permit2_data_to_eip712_json(Chain::Ethereum, data);
        assert_eq!(
            json,
            r#"{"domain":{"name":"Permit2","chainId":1,"verifyingContract":"0x000000000022D473030F116dDEE9F6B43aC78BA3"},"types":{"EIP712Domain":[{"name":"name","type":"string"},{"name":"chainId","type":"uint256"},{"name":"verifyingContract","type":"address"}],"PermitSingle":[{"name":"details","type":"PermitDetails"},{"name":"spender","type":"address"},{"name":"sigDeadline","type":"uint256"}],"PermitDetails":[{"name":"token","type":"address"},{"name":"amount","type":"uint160"},{"name":"expiration","type":"uint48"},{"name":"nonce","type":"uint48"}]},"primaryType":"PermitSingle","message":{"details":{"token":"0xdAC17F958D2ee523a2206206994597C13D831ec7","amount":"1461501637330902918203684832716283019655932542975","expiration":"1732780554","nonce":"0"},"spender":"0x3fC91A3afd70395Cd496C647d5a6CC9D4B2b7FAD","sigDeadline":"1730190354"}}"#
        );
    }
}
