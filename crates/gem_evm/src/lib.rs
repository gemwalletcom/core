use alloy_primitives::U256;
use std::str::FromStr;

pub mod across;
pub mod address;
pub mod call_decoder;
pub mod chainlink;
pub mod constants;
pub mod contracts;
pub mod eip712;
pub mod everstake;
pub mod ether_conv;
pub mod fee_calculator;
pub mod jsonrpc;
pub mod multicall3;
pub mod permit2;
#[cfg(feature = "rpc")]
pub mod registry;
pub mod thorchain;
pub mod uniswap;
pub mod weth;

pub mod models;

#[cfg(feature = "rpc")]
pub mod rpc;

#[cfg(feature = "rpc")]
pub mod provider;

pub use address::ethereum_address_checksum;
pub use eip712::{eip712_domain_types, EIP712Domain, EIP712Field, EIP712Type, EIP712TypedValue};

pub fn parse_u256(value: &str) -> Option<U256> {
    if let Some(stripped) = value.strip_prefix("0x") {
        U256::from_str_radix(stripped, 16).ok()
    } else {
        U256::from_str(value).ok()
    }
}
