use primitives::{AssetId, Chain, FiatQuoteType, FiatTransaction, FiatTransactionStatus};

use super::{client::PaybisClient, models::{Currency, PaybisTransaction}};

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


pub fn map_symbol_to_asset_id(symbol: &str) -> Option<AssetId> {
    match symbol {
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

pub fn map_order_from_response(transaction: PaybisTransaction) -> Result<FiatTransaction, Box<dyn std::error::Error + Send + Sync>> {
    let asset_id = map_symbol_to_asset_id(&transaction.crypto_currency);
    
    let status = match transaction.status.as_str() {
        "pending" => FiatTransactionStatus::Pending,
        "failed" | "cancelled" => FiatTransactionStatus::Failed,
        "completed" | "success" => FiatTransactionStatus::Complete,
        _ => FiatTransactionStatus::Unknown(transaction.status.clone()),
    };

    let transaction_type = FiatQuoteType::Buy; // TODO: Determine from API response

    Ok(FiatTransaction {
        asset_id,
        transaction_type,
        symbol: transaction.crypto_currency,
        provider_id: PaybisClient::NAME.id(),
        provider_transaction_id: transaction.id,
        status,
        country: transaction.country,
        fiat_amount: transaction.fiat_amount,
        fiat_currency: transaction.fiat_currency.to_uppercase(),
        transaction_hash: transaction.transaction_hash,
        address: transaction.wallet_address,
        fee_provider: transaction.service_fee,
        fee_network: transaction.network_fee,
        fee_partner: transaction.partner_fee,
    })
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
