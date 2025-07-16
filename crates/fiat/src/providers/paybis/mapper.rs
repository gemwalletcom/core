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
        _ => None,
    }
}

pub fn map_asset_chain(currency: &Currency) -> Option<Chain> {
    match currency.blockchain_name.as_deref()? {
        "ethereum" => Some(Chain::Ethereum),
        "binance-smart-chain" => Some(Chain::SmartChain),
        "binance-chain" => Some(Chain::SmartChain),
        "solana" => Some(Chain::Solana),
        "polygon" => Some(Chain::Polygon),
        "tron" => Some(Chain::Tron),
        "bitcoin" => Some(Chain::Bitcoin),
        "bitcoin-cash" => Some(Chain::BitcoinCash),
        "dogecoin" => Some(Chain::Doge),
        "litecoin" => Some(Chain::Litecoin),
        "ripple" => Some(Chain::Xrp),
        "ton" => Some(Chain::Ton),
        "stellar" => Some(Chain::Stellar),
        "polkadot" => Some(Chain::Polkadot),
        "cardano" => Some(Chain::Cardano),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_asset_chain() {
        assert_eq!(
            map_asset_chain(&Currency {
                code: "ETH".to_string(),
                blockchain_name: Some("ethereum".to_string()),
            }),
            Some(Chain::Ethereum)
        );

        assert_eq!(
            map_asset_chain(&Currency {
                code: "BTC".to_string(),
                blockchain_name: Some("bitcoin".to_string()),
            }),
            Some(Chain::Bitcoin)
        );

        assert_eq!(
            map_asset_chain(&Currency {
                code: "UNKNOWN".to_string(),
                blockchain_name: Some("unknown-chain".to_string()),
            }),
            None
        );

        assert_eq!(
            map_asset_chain(&Currency {
                code: "USD".to_string(),
                blockchain_name: None,
            }),
            None
        );
    }
}
