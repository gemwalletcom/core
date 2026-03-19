mod assets;
mod client;
mod contract;
mod mapper;
mod provider;

use assets::{YoAsset, supported_assets};
pub use provider::YoEarnProvider;

use alloy_primitives::{Address, address};

const YO_GATEWAY: Address = address!("0xF1EeE0957267b1A474323Ff9CfF7719E964969FA");
const YO_PARTNER_ID_GEM: u32 = 6548;
