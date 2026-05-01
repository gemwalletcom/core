mod client_factory;
mod error;
mod provider;
mod tonstakers;
mod yielder;
mod yo;

use std::{collections::HashMap, str::FromStr};

use primitives::{AssetId, YieldProvider};

pub use error::YielderError;
pub use provider::EarnProvider;
pub use yielder::Yielder;

pub fn get_underlying_assets_by_provider(provider_id: &str) -> HashMap<AssetId, Vec<AssetId>> {
    let Ok(provider) = YieldProvider::from_str(provider_id) else {
        return HashMap::new();
    };
    match provider {
        YieldProvider::Tonstakers => <tonstakers::TonstakersProvider as EarnProvider>::underlying_assets(),
        YieldProvider::Yo => <yo::YoEarnProvider as EarnProvider>::underlying_assets(),
    }
}
