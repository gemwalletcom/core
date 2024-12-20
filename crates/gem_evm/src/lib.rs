use alloy_primitives::U256;
use std::str::FromStr;

pub mod across;
pub mod address;
pub mod constants;
pub mod erc20;
pub mod erc2612;
pub mod ether_conv;
pub mod jsonrpc;
pub mod lido;
pub mod multicall3;
pub mod permit2;
pub mod uniswap;

pub fn parse_u256(value: &str) -> Option<U256> {
    if let Some(stripped) = value.strip_prefix("0x") {
        U256::from_str_radix(stripped, 16).ok()
    } else {
        U256::from_str(value).ok()
    }
}
