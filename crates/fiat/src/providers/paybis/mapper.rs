use primitives::{AssetId, Chain};

use super::model::Currency;

pub fn map_asset_id(currency: Currency) -> Option<AssetId> {
    if !currency.is_crypto() {
        return None;
    }
    match currency.code.as_str() {
        "BTC" => Some(AssetId::from_chain(Chain::Bitcoin)),
        "BCH" => Some(AssetId::from_chain(Chain::BitcoinCash)),
        "ETH" => Some(AssetId::from_chain(Chain::Ethereum)),
        "XRP" => Some(AssetId::from_chain(Chain::Xrp)),
        "SOL" => Some(AssetId::from_chain(Chain::Solana)),
        "XLM" => Some(AssetId::from_chain(Chain::Stellar)),
        "TRX" => Some(AssetId::from_chain(Chain::Tron)),
        "ADA" => Some(AssetId::from_chain(Chain::Cardano)),
        "OP" => Some(AssetId::from_chain(Chain::Optimism)),
        "LTC" => Some(AssetId::from_chain(Chain::Litecoin)),
        "ETH-BASE" => Some(AssetId::from_chain(Chain::Base)),
        "DOT" => Some(AssetId::from_chain(Chain::Polkadot)),
        "CELO" => Some(AssetId::from_chain(Chain::Celo)),
        _ => None,
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_asset_id() {
        assert_eq!(
            map_asset_id(Currency {
                code: "ETH".to_string(),
                blockchain_name: Some("ethereum".to_string()),
            }),
            Some(AssetId::from_chain(Chain::Ethereum))
        );

        assert_eq!(
            map_asset_id(Currency {
                code: "BTC".to_string(),
                blockchain_name: Some("bitcoin".to_string()),
            }),
            Some(AssetId::from_chain(Chain::Bitcoin))
        );

        assert_eq!(
            map_asset_id(Currency {
                code: "UNKNOWN".to_string(),
                blockchain_name: Some("unknown-chain".to_string()),
            }),
            None
        );

        assert_eq!(
            map_asset_id(Currency {
                code: "USD".to_string(),
                blockchain_name: None,
            }),
            None
        );
    }
}
