use super::{client::BanxaClient, models::Order};
use primitives::{AssetId, Chain, FiatQuoteType, FiatTransaction, FiatTransactionStatus};

pub fn map_asset_chain(chain: String) -> Option<Chain> {
    match chain.as_str() {
        "BTC" => Some(Chain::Bitcoin),
        "LTC" => Some(Chain::Litecoin),
        "ETH" => Some(Chain::Ethereum),
        "TRON" => Some(Chain::Tron),
        "BSC" | "BNB" => Some(Chain::SmartChain),
        "SOL" => Some(Chain::Solana),
        "MATIC" => Some(Chain::Polygon),
        "ATOM" => Some(Chain::Cosmos),
        "AVAX-C" => Some(Chain::AvalancheC),
        "XRP" => Some(Chain::Xrp),
        "FTM" => Some(Chain::Fantom),
        "DOGE" => Some(Chain::Doge),
        "APT" => Some(Chain::Aptos),
        "TON" => Some(Chain::Ton),
        "SUI" => Some(Chain::Sui),
        "NEAR" => Some(Chain::Near),
        "CELO" => Some(Chain::Celo),
        "THORCHAIN" => Some(Chain::Thorchain),
        "XLM" => Some(Chain::Stellar),
        "ADA" => Some(Chain::Cardano),
        "DOT" => Some(Chain::Polkadot),
        "ALGO" => Some(Chain::Algorand),
        "ZKSYNC" => Some(Chain::ZkSync),
        "BCH" => Some(Chain::BitcoinCash),
        "WLD" => Some(Chain::World),
        "OPTIMISM" => Some(Chain::Optimism),
        "LINEA" => Some(Chain::Linea),
        "UNICHAIN" => Some(Chain::Unichain),
        "ARB" => Some(Chain::Arbitrum),
        "BASE" => Some(Chain::Base),
        "S" => Some(Chain::Sonic),
        "INJ" => Some(Chain::Injective),
        "MNT" => Some(Chain::Mantle),
        _ => None,
    }
}

pub fn map_order(order: Order) -> Result<FiatTransaction, Box<dyn std::error::Error + Send + Sync>> {
    let chain = map_asset_chain(order.crypto.blockchain.clone());
    let asset_id = chain.map(AssetId::from_chain);

    let status = match order.status.as_str() {
        "pendingPayment" | "waitingPayment" | "paymentReceived" | "inProgress" | "coinTransferred" | "cryptoTransferred" | "extraVerification" => {
            FiatTransactionStatus::Pending
        }
        "cancelled" | "declined" | "expired" | "refunded" => FiatTransactionStatus::Failed,
        "complete" | "completed" | "succeeded" => FiatTransactionStatus::Complete,
        _ => FiatTransactionStatus::Unknown(order.status.clone()),
    };

    let transaction_type = match order.order_type.as_str() {
        "BUY" => FiatQuoteType::Buy,
        "SELL" => FiatQuoteType::Sell,
        _ => FiatQuoteType::Buy,
    };

    Ok(FiatTransaction {
        asset_id,
        transaction_type,
        symbol: order.crypto.id,
        provider_id: BanxaClient::NAME.id(),
        provider_transaction_id: order.id,
        status,
        country: order.country,
        fiat_amount: order.fiat_amount,
        fiat_currency: order.fiat,
        transaction_hash: order.tx_hash,
        address: Some(order.wallet_address),
        fee_provider: None,
        fee_network: order.network_fee,
        fee_partner: order.processing_fee,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::{FiatQuoteType, FiatTransactionStatus};

    #[test]
    fn test_map_order_sell_expired() {
        let response: Order = serde_json::from_str(include_str!("../../../testdata/banxa/transaction_buy_complete.json")).expect("Failed to parse test data");

        let result = map_order(response).expect("Failed to map order");

        assert_eq!(result.provider_id, "banxa");
        assert_eq!(result.provider_transaction_id, "1a0e15cbede2cd3776617683bd35b0f0");
        assert!(matches!(result.status, FiatTransactionStatus::Failed));
        assert!(matches!(result.transaction_type, FiatQuoteType::Sell));
        assert_eq!(result.symbol, "ETH");
        assert_eq!(result.fiat_currency, "USD");
        assert_eq!(result.fiat_amount, 3986.0);
        assert_eq!(result.country, None);
        assert_eq!(result.address, Some("".to_string()));
        assert!(result.asset_id.is_some()); // Should have asset ID for ETH
        assert_eq!(result.fee_network, Some(0.0));
        assert_eq!(result.fee_partner, Some(0.0));
    }
}
