use crate::model::{FiatProviderAsset, filter_token_id};
use primitives::currency::Currency;
use primitives::fiat_assets::FiatAssetLimits;
use primitives::{Chain, FiatProviderName, FiatTransactionStatus, FiatTransactionUpdate, PaymentType};

use super::models::{Asset, FiatCurrency, Order};

pub fn map_asset_chain(chain: &str) -> Option<Chain> {
    match chain {
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

pub fn map_order(order: Order) -> Result<FiatTransactionUpdate, Box<dyn std::error::Error + Send + Sync>> {
    match order.order_type.as_str() {
        "BUY" | "SELL" => {}
        _ => return Err(format!("Unsupported Banxa order type: {}", order.order_type).into()),
    }

    let provider_order_id = order.id.clone();
    let transaction_id = order.external_order_id.clone().unwrap_or_else(|| provider_order_id.clone());
    let provider_transaction_id = (transaction_id != provider_order_id).then_some(provider_order_id);

    Ok(FiatTransactionUpdate {
        transaction_id,
        provider_transaction_id,
        status: map_status(&order.status),
        transaction_hash: order.transaction_hash,
        address: (!order.wallet_address.is_empty()).then_some(order.wallet_address),
        fiat_amount: Some(order.fiat_amount),
        fiat_currency: Some(order.fiat.to_ascii_uppercase()),
    })
}

fn map_status(status: &str) -> FiatTransactionStatus {
    match status {
        "pendingPayment" | "waitingPayment" | "paymentReceived" | "inProgress" | "coinTransferred" | "cryptoTransferred" | "extraVerification" => FiatTransactionStatus::Pending,
        "cancelled" | "declined" | "expired" | "refunded" => FiatTransactionStatus::Failed,
        "complete" | "completed" | "succeeded" => FiatTransactionStatus::Complete,
        _ => FiatTransactionStatus::Unknown,
    }
}

fn map_asset_base(asset: Asset, buy_limits: Vec<FiatAssetLimits>, sell_limits: Vec<FiatAssetLimits>) -> Vec<FiatProviderAsset> {
    let symbol = asset.id.clone();
    let is_buy_enabled = !buy_limits.is_empty();
    let is_sell_enabled = !sell_limits.is_empty();

    asset
        .blockchains
        .into_iter()
        .map(|blockchain| {
            let chain = map_asset_chain(blockchain.id.as_str());
            let token_id = filter_token_id(chain, blockchain.address);

            FiatProviderAsset {
                id: format!("{symbol}-{}", blockchain.id),
                provider: FiatProviderName::Banxa,
                chain,
                symbol: symbol.clone(),
                token_id,
                network: Some(blockchain.id),
                enabled: true,
                is_buy_enabled,
                is_sell_enabled,
                unsupported_countries: Some(blockchain.unsupported_countries.list_map()),
                buy_limits: buy_limits.clone(),
                sell_limits: sell_limits.clone(),
            }
        })
        .collect()
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
                    let payment_type = map_payment_type(payment_method.id.as_str())?;

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
    use crate::providers::banxa::models::{FiatCurrency, Order};
    use primitives::currency::Currency;
    use primitives::{FiatTransactionStatus, PaymentType};

    use super::{map_limits, map_order};

    #[test]
    fn map_order_maps_sell_failure() {
        let response: Order = serde_json::from_str(include_str!("../../../testdata/banxa/transaction_sell_failed.json")).unwrap();
        let result = map_order(response).unwrap();

        assert_eq!(result.transaction_id, "123");
        assert_eq!(result.provider_transaction_id, None);
        assert_eq!(result.status, FiatTransactionStatus::Failed);
        assert_eq!(result.fiat_amount, Some(595.3));
        assert_eq!(result.fiat_currency, Some("USD".to_string()));
        assert_eq!(result.address, Some("0x123".to_string()));
        assert_eq!(result.transaction_hash, None);
    }

    #[test]
    fn map_order_prefers_external_order_id_for_reconciliation() {
        let result = map_order(Order {
            id: "banxa_order_123".to_string(),
            external_order_id: Some("quote_123".to_string()),
            status: "completed".to_string(),
            fiat: "usd".to_string(),
            fiat_amount: 100.0,
            wallet_address: "bc1qexample".to_string(),
            transaction_hash: Some("tx_hash".to_string()),
            order_type: "BUY".to_string(),
        })
        .unwrap();

        assert_eq!(result.transaction_id, "quote_123");
        assert_eq!(result.provider_transaction_id, Some("banxa_order_123".to_string()));
        assert_eq!(result.status, FiatTransactionStatus::Complete);
        assert_eq!(result.fiat_amount, Some(100.0));
        assert_eq!(result.fiat_currency, Some("USD".to_string()));
        assert_eq!(result.address, Some("bc1qexample".to_string()));
        assert_eq!(result.transaction_hash, Some("tx_hash".to_string()));
    }

    #[test]
    fn map_limits_maps_supported_payment_methods() {
        let fiat_currencies: Vec<FiatCurrency> = serde_json::from_str(include_str!("../../../testdata/banxa/fiat_currencies.json")).unwrap();
        let buy_limits = map_limits(&fiat_currencies);

        assert_eq!(buy_limits.len(), 4);

        let eur_card_limit = buy_limits
            .iter()
            .find(|limit| limit.currency == Currency::EUR && limit.payment_type == PaymentType::Card)
            .unwrap();
        assert_eq!(eur_card_limit.min_amount, Some(20.0));
        assert_eq!(eur_card_limit.max_amount, Some(15000.0));

        let usd_google_pay_limit = buy_limits
            .iter()
            .find(|limit| limit.currency == Currency::USD && limit.payment_type == PaymentType::GooglePay)
            .unwrap();
        assert_eq!(usd_google_pay_limit.min_amount, Some(20.0));
        assert_eq!(usd_google_pay_limit.max_amount, Some(15000.0));
    }
}
