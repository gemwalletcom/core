use alloy_primitives::U256;
use std::str::FromStr;

pub mod across;
pub mod address;
pub mod call_decoder;
pub mod chainlink;
pub mod constants;
pub mod contracts;
pub mod domain;
pub mod eip712;
pub mod ether_conv;
pub mod everstake;
pub mod fee_calculator;
pub mod jsonrpc;
pub mod message;
pub mod monad;
pub mod multicall3;
pub mod permit2;
#[cfg(feature = "rpc")]
pub mod registry;
#[cfg(feature = "signer")]
pub mod signer;
pub mod siwe;
pub mod thorchain;
pub mod u256;
pub mod uniswap;
pub mod weth;

pub mod models;

#[cfg(feature = "rpc")]
pub mod rpc;

#[cfg(feature = "rpc")]
pub mod provider;

#[cfg(any(test, feature = "testkit"))]
pub mod testkit;

pub const ETHEREUM_MESSAGE_PREFIX: &str = "\x19Ethereum Signed Message:\n";
pub const SIGNATURE_LENGTH: usize = 65;
pub const RECOVERY_ID_INDEX: usize = SIGNATURE_LENGTH - 1;
pub const ETHEREUM_RECOVERY_ID_OFFSET: u8 = 27;

pub use address::ethereum_address_checksum;
pub use eip712::{EIP712Domain, EIP712Field, EIP712Type, EIP712TypedValue, eip712_domain_types};

pub fn parse_u256(value: &str) -> Option<U256> {
    if let Some(stripped) = value.strip_prefix("0x") {
        U256::from_str_radix(stripped, 16).ok()
    } else {
        U256::from_str(value).ok()
    }
}
