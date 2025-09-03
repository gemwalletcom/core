use crate::{Asset, Chain, FiatBuyQuote};

impl FiatBuyQuote {
    pub fn mock() -> Self {
        FiatBuyQuote {
            asset: Asset::from_chain(Chain::Bitcoin),
            asset_id: Chain::Bitcoin.as_asset_id().to_string(),
            ip_address: "192.168.1.1".to_string(),
            fiat_currency: "USD".to_string(),
            fiat_amount: 100.0,
            fiat_value: "100.0".to_string(),
            wallet_address: "bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh".to_string(),
        }
    }
}
