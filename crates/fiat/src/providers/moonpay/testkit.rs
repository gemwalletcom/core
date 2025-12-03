use super::models::{Asset, CurrencyMetadata, FiatCurrencyType};

impl Asset {
    pub fn mock(code: &str, network_code: &str, contract_address: Option<&str>, is_base_asset: bool) -> Self {
        Self {
            code: code.to_string(),
            metadata: Some(CurrencyMetadata {
                contract_address: contract_address.map(|s| s.to_string()),
                network_code: network_code.to_string(),
            }),
            is_suspended: Some(false),
            is_base_asset: Some(is_base_asset),
            not_allowed_countries: None,
            currency_type: FiatCurrencyType::Crypto,
            min_buy_amount: None,
            max_buy_amount: None,
            min_sell_amount: None,
            max_sell_amount: None,
        }
    }
}
