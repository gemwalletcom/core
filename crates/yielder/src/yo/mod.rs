mod api;
mod client;
mod contract;
mod error;
mod model;
mod provider;
mod vault;

pub use api::{YoApiClient, YoPerformanceData};
pub use client::{YoGatewayClient, YoProvider};
pub use contract::{IYoGateway, IYoVaultToken};
pub use error::YieldError;
pub use model::PositionData;
pub use provider::YoYieldProvider;
pub use vault::{YO_USD, YO_USDT, YoVault, vaults};

use alloy_primitives::{Address, address};

pub const YO_GATEWAY: Address = address!("0xF1EeE0957267b1A474323Ff9CfF7719E964969FA");
pub const YO_PARTNER_ID_GEM: u32 = 6548;
