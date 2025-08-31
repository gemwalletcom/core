use primitives::{AssetId, Chain, FiatQuoteType, FiatTransaction, FiatTransactionStatus};
use super::{client::TransakClient, models::{Asset, TransakOrderResponse}};

pub fn map_asset_chain(asset: Asset) -> Option<Chain> {
    match asset.network.name.as_str() {
        "ethereum" => Some(Chain::Ethereum),
        "polygon" => Some(Chain::Polygon),
        "aptos" => Some(Chain::Aptos),
        "sui" => Some(Chain::Sui),
        "arbitrum" => Some(Chain::Arbitrum),
        "optimism" => Some(Chain::Optimism),
        "base" => Some(Chain::Base),
        "bsc" => Some(Chain::SmartChain),
        "tron" => Some(Chain::Tron),
        "solana" => Some(Chain::Solana),
        "avaxcchain" => Some(Chain::AvalancheC),
        "ton" => Some(Chain::Ton),
        "osmosis" => Some(Chain::Osmosis),
        "fantom" => Some(Chain::Fantom),
        "injective" => Some(Chain::Injective),
        "sei" => Some(Chain::Sei),
        "linea" => Some(Chain::Linea),
        "zksync" => Some(Chain::ZkSync),
        "celo" => Some(Chain::Celo),
        "mantle" => Some(Chain::Mantle),
        "opbnb" => Some(Chain::OpBNB),
        "unichain" => Some(Chain::Unichain),
        "stellar" => Some(Chain::Stellar),
        "algorand" => Some(Chain::Algorand),
        "berachain" => Some(Chain::Berachain),
        "hyperevm" => Some(Chain::Hyperliquid),
        "hyperliquid" => Some(Chain::HyperCore),
        "mainnet" => match asset.coin_id.as_str() {
            "bitcoin" => Some(Chain::Bitcoin),
            "litecoin" => Some(Chain::Litecoin),
            "ripple" => Some(Chain::Xrp),
            "dogecoin" => Some(Chain::Doge),
            "tron" => Some(Chain::Tron),
            "cosmos" => Some(Chain::Cosmos),
            "near" => Some(Chain::Near),
            "stellar" => Some(Chain::Stellar),
            "algorand" => Some(Chain::Algorand),
            "polkadot" => Some(Chain::Polkadot),
            "cardano" => Some(Chain::Cardano),
            _ => None,
        },
        _ => None,
    }
}

fn map_status(status: &str) -> FiatTransactionStatus {
    match status {
        "ORDER_PAYMENT_VERIFYING" | "PAYMENT_DONE_MARKED_BY_USER" | "PENDING_DELIVERY_FROM_TRANSAK" | "AWAITING_PAYMENT_FROM_USER" => {
            FiatTransactionStatus::Pending
        }
        "EXPIRED" | "FAILED" | "CANCELLED" | "REFUNDED" => FiatTransactionStatus::Failed,
        "COMPLETED" => FiatTransactionStatus::Complete,
        _ => FiatTransactionStatus::Unknown(status.to_string()),
    }
}

fn map_transaction_type(transaction_type: &str) -> FiatQuoteType {
    match transaction_type {
        "BUY" => FiatQuoteType::Buy,
        "SELL" => FiatQuoteType::Sell,
        _ => FiatQuoteType::Buy,
    }
}

pub fn map_network_to_chain(network: &str) -> Option<Chain> {
    match network {
        "ethereum" => Some(Chain::Ethereum),
        "polygon" => Some(Chain::Polygon),
        "aptos" => Some(Chain::Aptos),
        "sui" => Some(Chain::Sui),
        "arbitrum" => Some(Chain::Arbitrum),
        "optimism" => Some(Chain::Optimism),
        "base" => Some(Chain::Base),
        "bsc" => Some(Chain::SmartChain),
        "tron" => Some(Chain::Tron),
        "solana" => Some(Chain::Solana),
        "avaxcchain" => Some(Chain::AvalancheC),
        "ton" => Some(Chain::Ton),
        "osmosis" => Some(Chain::Osmosis),
        "fantom" => Some(Chain::Fantom),
        "injective" => Some(Chain::Injective),
        "sei" => Some(Chain::Sei),
        "linea" => Some(Chain::Linea),
        "zksync" => Some(Chain::ZkSync),
        "celo" => Some(Chain::Celo),
        "mantle" => Some(Chain::Mantle),
        "opbnb" => Some(Chain::OpBNB),
        "unichain" => Some(Chain::Unichain),
        "stellar" => Some(Chain::Stellar),
        "algorand" => Some(Chain::Algorand),
        "berachain" => Some(Chain::Berachain),
        "hyperevm" => Some(Chain::Hyperliquid),
        "hyperliquid" => Some(Chain::HyperCore),
        "mainnet" => {
            // For mainnet networks, we need to infer from symbol
            // This would need more logic based on the crypto_currency
            None
        },
        _ => None,
    }
}

pub fn map_order_from_response(payload: TransakOrderResponse) -> Result<FiatTransaction, Box<dyn std::error::Error + Send + Sync>> {
    let chain = map_network_to_chain(&payload.network);
    let asset_id = chain.map(AssetId::from_chain);

    Ok(FiatTransaction {
        asset_id,
        transaction_type: map_transaction_type(&payload.is_buy_or_sell),
        symbol: payload.crypto_currency,
        provider_id: TransakClient::NAME.id(),
        provider_transaction_id: payload.id,
        status: map_status(&payload.status),
        country: payload.country_code,
        fiat_amount: payload.fiat_amount,
        fiat_currency: payload.fiat_currency,
        transaction_hash: payload.transaction_hash,
        address: payload.wallet_address,
        fee_provider: None, // Not available in order response
        fee_network: None,
        fee_partner: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::transak::models::{Data, TransakOrderResponse};
    use primitives::{FiatQuoteType, FiatTransactionStatus};

    #[test]
    fn test_map_order_buy_failed() {
        let response: Data<TransakOrderResponse> = serde_json::from_str(include_str!("../../../testdata/transak/transaction_buy_error.json")).expect("Failed to parse test data");

        let result = map_order_from_response(response.data).expect("Failed to map order");

        assert_eq!(result.provider_id, "transak");
        assert_eq!(result.provider_transaction_id, "df7997b7-a19f-447e-b9fe-2f0eb7cb7b3a");
        assert!(matches!(result.status, FiatTransactionStatus::Failed));
        assert!(matches!(result.transaction_type, FiatQuoteType::Buy));
        assert_eq!(result.symbol, "SUI");
        assert_eq!(result.fiat_currency, "USD");
        assert_eq!(result.fiat_amount, 108.0);
        assert_eq!(result.country, Some("DK".to_string()));
        assert_eq!(result.address, Some("0xf47abc9e2ed94fb555b3e31e08ad8aa8ac64eea0ff15ba0e7b443cef4aaabffe".to_string()));
        assert!(result.asset_id.is_some()); // Should have asset ID for SUI
    }
}
