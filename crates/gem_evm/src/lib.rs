use alloy_primitives::U256;
use std::str::FromStr;

pub mod across;
pub mod address;
pub mod chainlink;
pub mod eip712;
pub mod erc20;
pub mod erc2612;
pub mod ether_conv;
pub mod jsonrpc;
pub mod lido;
pub mod multicall3;
pub mod permit2;
pub mod registry;
pub mod thorchain;
pub mod uniswap;
pub mod weth;

#[cfg(feature = "typeshare")]
pub mod typeshare;

#[cfg(feature = "rpc")]
pub mod rpc;

pub use address::ethereum_address_checksum;
pub use eip712::{eip712_domain_types, EIP712Domain, EIP712Field, EIP712Type, EIP712TypedValue};

pub fn parse_u256(value: &str) -> Option<U256> {
    if let Some(stripped) = value.strip_prefix("0x") {
        U256::from_str_radix(stripped, 16).ok()
    } else {
        U256::from_str(value).ok()
    }
}
