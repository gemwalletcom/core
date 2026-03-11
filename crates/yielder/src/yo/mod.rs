mod assets;
mod client;
mod contract;
mod provider;

pub use assets::{YoAsset, supported_assets};
pub use client::{YoClient, YoGatewayClient};
pub use provider::YoEarnProvider;

use alloy_primitives::{Address, address};

pub const YO_GATEWAY: Address = address!("0xF1EeE0957267b1A474323Ff9CfF7719E964969FA");
pub const YO_PARTNER_ID_GEM: u32 = 6548;
