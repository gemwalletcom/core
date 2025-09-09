use super::{
    client::BanxaClient,
    models::{Asset, FiatCurrency, Order},
};
use crate::model::{filter_token_id, FiatProviderAsset};
use primitives::currency::Currency;
use primitives::fiat_assets::FiatAssetLimits;
use primitives::PaymentType;
use primitives::{AssetId, Chain, FiatProviderName, FiatQuoteType, FiatTransaction, FiatTransactionStatus};

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
    })
}

fn map_asset_base(asset: Asset, buy_limits: Vec<FiatAssetLimits>, sell_limits: Vec<FiatAssetLimits>) -> Vec<FiatProviderAsset> {
    let asset_id = asset.id.clone();
    asset
        .blockchains
        .into_iter()
        .map(|blockchain| {
            let chain = map_asset_chain(blockchain.clone().id.clone());
            let token_id = filter_token_id(chain, blockchain.clone().address);
            let id = asset_id.clone() + "-" + blockchain.clone().id.as_str();
            FiatProviderAsset {
                id,
                provider: FiatProviderName::Banxa,
                chain,
                token_id,
                symbol: asset_id.clone(),
                network: Some(blockchain.id),
                enabled: true,
                unsupported_countries: Some(blockchain.unsupported_countries.list_map()),
                buy_limits: buy_limits.clone(),
                sell_limits: sell_limits.clone(),
            }
        })
        .collect()
}

pub fn map_asset(asset: Asset) -> Vec<FiatProviderAsset> {
    map_asset_base(asset, vec![], vec![])
}

pub fn map_asset_with_limits(asset: Asset, buy_fiat_currencies: &[FiatCurrency], sell_fiat_currencies: &[FiatCurrency]) -> Vec<FiatProviderAsset> {
    map_asset_base(asset, map_limits(buy_fiat_currencies), map_limits(sell_fiat_currencies))
}

fn map_limits(fiat_currencies: &[FiatCurrency]) -> Vec<FiatAssetLimits> {
    fiat_currencies
        .iter()
        .filter_map(|fiat_currency| fiat_currency.id.parse::<Currency>().ok().map(|currency| (currency, fiat_currency)))
        .flat_map(|(currency, fiat_currency)| {
            fiat_currency
                .supported_payment_methods
                .iter()
                .filter_map(|payment_method| {
                    let payment_type = map_payment_type(&payment_method.id)?;
                    Some(FiatAssetLimits {
                        currency: currency.clone(),
                        payment_type,
                        min_amount: Some(payment_method.minimum),
                        max_amount: Some(payment_method.maximum),
                    })
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

fn map_payment_type(payment_id: &str) -> Option<PaymentType> {
    match payment_id {
        "debit-credit-card" => Some(PaymentType::Card),
        "google-pay" => Some(PaymentType::GooglePay),
        "apple-pay" => Some(PaymentType::ApplePay),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::currency::Currency;
    use primitives::{FiatQuoteType, FiatTransactionStatus, PaymentType};

    #[test]
    fn test_map_order_sell_expired() {
        let response: Order = serde_json::from_str(include_str!("../../../testdata/banxa/transaction_buy_complete.json")).expect("Failed to parse test data");

        let result = map_order(response).expect("Failed to map order");

        assert_eq!(result.provider_id, "banxa");
        assert_eq!(result.provider_transaction_id, "test");
        assert!(matches!(result.status, FiatTransactionStatus::Failed));
        assert!(matches!(result.transaction_type, FiatQuoteType::Sell));
        assert_eq!(result.symbol, "ETH");
        assert_eq!(result.fiat_currency, "USD");
        assert_eq!(result.fiat_amount, 3986.0);
        assert!(result.asset_id.is_some());
    }

    #[test]
    fn test_map_order_sell_failed() {
        let response: Order = serde_json::from_str(include_str!("../../../testdata/banxa/transaction_sell_failed.json")).expect("Failed to parse test data");

        let result = map_order(response).expect("Failed to map order");

        assert_eq!(result.provider_id, "banxa");
        assert_eq!(result.provider_transaction_id, "123");
        assert!(matches!(result.status, FiatTransactionStatus::Failed));
        assert!(matches!(result.transaction_type, FiatQuoteType::Sell));
        assert_eq!(result.symbol, "ETH");
        assert_eq!(result.fiat_currency, "USD");
        assert_eq!(result.fiat_amount, 595.3);
        assert_eq!(result.country, None);
        assert_eq!(result.address, Some("0x123".to_string()));
        assert!(result.asset_id.is_some());
    }

    #[test]
    fn test_map_limits() {
        let fiat_currencies: Vec<FiatCurrency> = serde_json::from_str(include_str!("../../../testdata/banxa/fiat_currencies.json")).unwrap();

        let buy_limits = map_limits(&fiat_currencies);
        let sell_limits = map_limits(&fiat_currencies);

        assert_eq!(buy_limits.len(), 4); // 2 EUR + 2 USD payment methods (sepa-bank-transfer not supported)
        assert_eq!(sell_limits.len(), 4);

        let eur_card_limit = buy_limits
            .iter()
            .find(|limit| limit.currency == Currency::EUR && limit.payment_type == PaymentType::Card)
            .unwrap();
        assert_eq!(eur_card_limit.min_amount, Some(20.0));
        assert_eq!(eur_card_limit.max_amount, Some(15000.0));

        let usd_google_pay_limit = sell_limits
            .iter()
            .find(|limit| limit.currency == Currency::USD && limit.payment_type == PaymentType::GooglePay)
            .unwrap();
        assert_eq!(usd_google_pay_limit.min_amount, Some(20.0));
        assert_eq!(usd_google_pay_limit.max_amount, Some(15000.0));
    }
}
