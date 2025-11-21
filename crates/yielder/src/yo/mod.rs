mod client;
mod contract;
mod error;
mod provider;
mod vault;

pub use client::{YoGatewayApi, YoGatewayClient};
pub use contract::IYoGateway;
pub use error::YieldError;
pub use provider::YoYieldProvider;
pub use vault::{vaults, YoVault, YO_ETH, YO_USD};

use alloy_primitives::{address, Address};

pub const YO_GATEWAY_BASE_MAINNET: Address = address!("0xF1EeE0957267b1A474323Ff9CfF7719E964969FA");
pub const YO_PARTNER_ID_GEM: u32 = 6548;
