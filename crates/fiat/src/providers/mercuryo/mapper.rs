use std::collections::HashMap;

use super::models::{Asset, CurrencyLimits, WebhookData};
use crate::{model::FiatProviderAsset, providers::mercuryo::models::FiatPaymentMethod};
use primitives::{Chain, FiatProviderName, FiatTransactionStatus, FiatTransactionUpdate};
use primitives::{PaymentType, currency::Currency, fiat_assets::FiatAssetLimits};

use super::models::Quote;

pub fn map_sell_quote(buy_quote: Quote, sell_quote: Quote, requested_fiat_amount: f64) -> Quote {
    let fee_ratio = sell_quote.fiat_amount / requested_fiat_amount;
    let adjusted_crypto_amount = buy_quote.amount / fee_ratio;

    Quote {
        amount: adjusted_crypto_amount,
        currency: sell_quote.currency,
        fiat_amount: requested_fiat_amount,
    }
}

pub fn map_asset_chain(chain: String) -> Option<Chain> {
    match chain.as_str() {
        "BITCOIN" => Some(Chain::Bitcoin),
        "ETHEREUM" => Some(Chain::Ethereum),
        "OPTIMISM" => Some(Chain::Optimism),
        "ARBITRUM" => Some(Chain::Arbitrum),
        "BASE" => Some(Chain::Base),
        "TRON" => Some(Chain::Tron),
        "BINANCESMARTCHAIN" => Some(Chain::SmartChain),
        "SOLANA" => Some(Chain::Solana),
        "POLYGON" => Some(Chain::Polygon),
        "COSMOS" => Some(Chain::Cosmos),
        "AVALANCHE" => Some(Chain::AvalancheC),
        "RIPPLE" => Some(Chain::Xrp),
        "LITECOIN" => Some(Chain::Litecoin),
        "FANTOM" => Some(Chain::Fantom),
        "DOGECOIN" => Some(Chain::Doge),
        "CELESTIA" => Some(Chain::Celestia),
        "NEWTON" => Some(Chain::Ton),
        "NEAR_PROTOCOL" => Some(Chain::Near),
        "LINEA" => Some(Chain::Linea),
        "ZKSYNC" => Some(Chain::ZkSync),
        "INJECTIVE" => Some(Chain::Injective),
        "STELLAR" => Some(Chain::Stellar),
        "ALGORAND" => Some(Chain::Algorand),
        "POLKADOT" => Some(Chain::Polkadot),
        "CARDANO" => Some(Chain::Cardano),
        "BITCOINCASH" => Some(Chain::BitcoinCash),
        "SUI" => Some(Chain::Sui),
        "SONIC" => Some(Chain::Sonic),
        "MONAD" => Some(Chain::Monad),
        _ => None,
    }
}

fn map_limits(fiat_payment_methods: &HashMap<String, FiatPaymentMethod>) -> Vec<FiatAssetLimits> {
    fiat_payment_methods
        .iter()
        .filter_map(|(currency_code, fiat_method)| {
            let currency = currency_code.parse::<Currency>().ok()?;
            Some((currency, fiat_method))
        })
        .flat_map(|(currency, fiat_method)| {
            fiat_method
                .payment_methods
                .iter()
                .filter_map(|payment_method| {
                    let payment_type = map_payment_type(&payment_method.code, &payment_method.name)?;
                    Some(FiatAssetLimits {
                        currency: currency.clone(),
                        payment_type,
                        min_amount: Some(fiat_method.limits.min),
                        max_amount: Some(fiat_method.limits.max),
                    })
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

fn map_payment_type(payment_code: &str, payment_name: &str) -> Option<PaymentType> {
    match payment_code {
        "card" if payment_name == "Visa" => Some(PaymentType::Card),
        "google" => Some(PaymentType::GooglePay),
        "apple" => Some(PaymentType::ApplePay),
        _ => None,
    }
}

pub fn map_order_from_webhook(webhook: WebhookData) -> FiatTransactionUpdate {
    let WebhookData {
        id,
        merchant_transaction_id,
        status,
        fiat_amount,
        fiat_currency,
        tx,
    } = webhook;
    let transaction_id = merchant_transaction_id.unwrap_or_else(|| id.clone());
    let provider_transaction_id = (transaction_id != id).then_some(id);

    FiatTransactionUpdate {
        transaction_id,
        provider_transaction_id,
        status: map_status(&status),
        transaction_hash: tx.and_then(|tx| tx.id),
        fiat_amount: Some(fiat_amount),
        fiat_currency: Some(fiat_currency.to_ascii_uppercase()),
    }
}

fn map_status(status: &str) -> FiatTransactionStatus {
    match status {
        "new" | "pending" | "order_scheduled" => FiatTransactionStatus::Pending,
        "cancelled" | "order_failed" | "descriptor_failed" => FiatTransactionStatus::Failed,
        "paid" | "completed" | "succeeded" => FiatTransactionStatus::Complete,
        _ => FiatTransactionStatus::Unknown,
    }
}

fn map_asset_base(asset: Asset, buy_limits: Vec<FiatAssetLimits>, sell_limits: Vec<FiatAssetLimits>) -> Option<FiatProviderAsset> {
    let chain = map_asset_chain(asset.network.clone());
    let token_id = if asset.contract.is_empty() { None } else { Some(asset.contract.clone()) };
    let is_buy_enabled = !buy_limits.is_empty();
    let is_sell_enabled = !sell_limits.is_empty();
    Some(FiatProviderAsset {
        id: asset.clone().currency + "_" + asset.network.as_str(),
        provider: FiatProviderName::Mercuryo,
        chain,
        token_id,
        symbol: asset.clone().currency,
        network: Some(asset.network),
        enabled: true,
        is_buy_enabled,
        is_sell_enabled,
        unsupported_countries: None,
        buy_limits,
        sell_limits,
    })
}

pub fn map_asset(asset: Asset) -> Option<FiatProviderAsset> {
    map_asset_base(asset, vec![], vec![])
}

pub fn map_asset_with_limits(asset: Asset, buy_limits: Vec<FiatAssetLimits>, sell_limits: Vec<FiatAssetLimits>) -> Option<FiatProviderAsset> {
    map_asset_base(asset, buy_limits, sell_limits)
}

pub fn map_asset_limits(currency_limits: Option<&CurrencyLimits>, currency: Currency, fiat_payment_methods: &HashMap<String, FiatPaymentMethod>) -> Vec<FiatAssetLimits> {
    match currency_limits {
        Some(limits) => vec![FiatAssetLimits {
            currency,
            payment_type: PaymentType::Card,
            min_amount: Some(limits.min),
            max_amount: Some(limits.max),
        }],
        None => map_limits(fiat_payment_methods),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::mercuryo::models::{Currencies, Response, Webhook};
    use primitives::FiatTransactionStatus;

    #[test]
    fn test_map_order_from_webhook() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let webhook: Webhook = serde_json::from_str(include_str!("../../../testdata/mercuryo/webhook_buy_complete.json"))?;

        let result = map_order_from_webhook(webhook.data);

        assert_eq!(
            result,
            FiatTransactionUpdate {
                transaction_id: "11111111-2222-4333-8444-555555555555".to_string(),
                provider_transaction_id: Some("buy_provider_tx_123456789".to_string()),
                status: FiatTransactionStatus::Failed,
                transaction_hash: None,
                fiat_amount: Some(270.0),
                fiat_currency: Some("USD".to_string()),
            }
        );

        Ok(())
    }

    #[test]
    fn test_map_order_from_withdraw_webhook() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let webhook: Webhook = serde_json::from_str(include_str!("../../../testdata/mercuryo/webhook_withdraw_complete.json"))?;

        let result = map_order_from_webhook(webhook.data);

        assert_eq!(
            result,
            FiatTransactionUpdate {
                transaction_id: "aaaaaaaa-bbbb-4ccc-8ddd-eeeeeeeeeeee".to_string(),
                provider_transaction_id: Some("withdraw_provider_tx_123456789".to_string()),
                status: FiatTransactionStatus::Complete,
                transaction_hash: Some("CELESTIA_TX_HASH_123".to_string()),
                fiat_amount: Some(39.23),
                fiat_currency: Some("EUR".to_string()),
            }
        );

        Ok(())
    }

    #[test]
    fn test_map_trump_asset() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let currencies = serde_json::from_str::<Response<Currencies>>(include_str!("../../../testdata/mercuryo/assets.json"))?.data;

        let trump_asset = currencies
            .config
            .crypto_currencies
            .iter()
            .find(|asset| asset.currency == "TRUMP" && asset.network == "SOLANA")
            .unwrap();

        let result = map_asset_with_limits(trump_asset.clone(), vec![], vec![]).unwrap();

        assert_eq!(result.symbol, "TRUMP");
        assert_eq!(result.chain, Some(Chain::Solana));
        assert_eq!(result.token_id, Some("6p6xgHyF7AeE6TZkSmFsko444wqoP15icUSqi2jfGiPN".to_string()));
        assert_eq!(result.network, Some("SOLANA".to_string()));
        assert_eq!(result.id, "TRUMP_SOLANA");

        Ok(())
    }

    #[test]
    fn test_contract_assets_mapping() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let currencies = serde_json::from_str::<Response<Currencies>>(include_str!("../../../testdata/mercuryo/assets.json"))?.data;

        let all_assets: Vec<_> = currencies
            .config
            .crypto_currencies
            .into_iter()
            .flat_map(|asset| map_asset_with_limits(asset, vec![], vec![]))
            .collect();

        let contract_assets: Vec<_> = all_assets.iter().filter(|asset| asset.token_id.is_some()).collect();

        let trump_assets: Vec<_> = all_assets.iter().filter(|asset| asset.symbol == "TRUMP").collect();

        println!("Total assets: {}", all_assets.len());
        println!("Contract-based assets: {}", contract_assets.len());
        println!("TRUMP assets: {}", trump_assets.len());

        for trump in &trump_assets {
            println!("TRUMP asset: {} on {:?} with contract: {:?}", trump.id, trump.chain, trump.token_id);
            println!("TRUMP asset_id(): {:?}", trump.asset_id());
        }

        assert!(!contract_assets.is_empty());
        assert!(!trump_assets.is_empty());

        Ok(())
    }

    #[test]
    fn test_map_sell_quote() {
        let buy_quote = Quote {
            amount: 0.031198,
            currency: "ETH".to_string(),
            fiat_amount: 100.0,
        };
        let sell_quote = Quote {
            amount: 0.031198,
            currency: "ETH".to_string(),
            fiat_amount: 90.26,
        };

        let result = map_sell_quote(buy_quote, sell_quote, 100.0);

        assert!((result.amount - 0.03456).abs() < 0.0001);
        assert_eq!(result.fiat_amount, 100.0);
        assert_eq!(result.currency, "ETH");
    }
}
