use crate::{Asset, Chain, PerpetualConfirmData, PerpetualDirection};

impl PerpetualConfirmData {
    pub fn mock(direction: PerpetualDirection, asset_index: u32) -> Self {
        Self {
            direction,
            base_asset: Asset::from_chain(Chain::HyperCore),
            asset_index: asset_index as i32,
            price: "123.45".to_string(),
            fiat_value: 100.0,
            size: "2.5".to_string(),
            slippage: 0.01,
            leverage: 5,
            pnl: None,
            entry_price: None,
            market_price: 123.45,
            margin_amount: 50.0,
        }
    }
}
