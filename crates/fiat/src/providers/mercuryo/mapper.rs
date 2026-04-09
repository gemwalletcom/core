use std::collections::HashMap;

use super::models::{
    Asset, BuyAcquirerTransaction, BuyTransaction, CurrencyLimits, DepositTransaction, MercuryoTransactionResponse, MobilePayTransaction, SellTransaction, WebhookData,
    WithdrawTransaction,
};
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

pub fn map_order_from_response(transaction: MercuryoTransactionResponse) -> Result<FiatTransactionUpdate, Box<dyn std::error::Error + Send + Sync>> {
    if let Some(buy) = transaction.buy {
        return Ok(map_buy_transaction(buy, transaction.withdraw));
    }

    if let Some(buy_acquirer) = transaction.buy_acquirer {
        return Ok(map_buy_acquirer_transaction(buy_acquirer, transaction.withdraw));
    }

    if let Some(mobile_pay) = transaction.mobile_pay {
        return Ok(map_mobile_pay_transaction(mobile_pay, transaction.withdraw));
    }

    if let Some(sell) = transaction.sell {
        return Ok(map_sell_transaction_new(sell, transaction.deposit));
    }

    if let Some(withdraw) = transaction.withdraw {
        return Ok(map_sell_transaction(withdraw));
    }

    Err("No valid transaction data found".into())
}

pub fn map_order_from_webhook(webhook: WebhookData) -> FiatTransactionUpdate {
    FiatTransactionUpdate {
        transaction_id: webhook.merchant_transaction_id.unwrap_or(webhook.id),
        provider_transaction_id: None,
        status: map_status(&webhook.status, None),
        transaction_hash: None,
        fiat_amount: Some(webhook.fiat_amount),
        fiat_currency: Some(webhook.fiat_currency.to_ascii_uppercase()),
    }
}

fn map_status(status: &str, withdraw_status: Option<&str>) -> FiatTransactionStatus {
    match status {
        "new" | "pending" | "order_scheduled" => FiatTransactionStatus::Pending,
        "cancelled" | "order_failed" | "descriptor_failed" => FiatTransactionStatus::Failed,
        "paid" => {
            if let Some(w_status) = withdraw_status {
                match w_status {
                    "completed" => FiatTransactionStatus::Complete,
                    "cancelled" | "failed" => FiatTransactionStatus::Failed,
                    _ => FiatTransactionStatus::Pending,
                }
            } else {
                FiatTransactionStatus::Complete
            }
        }
        "completed" | "succeeded" => FiatTransactionStatus::Complete,
        _ => FiatTransactionStatus::Unknown,
    }
}

fn map_buy_like_transaction(
    status: &str,
    merchant_transaction_id: String,
    fiat_amount: f64,
    fiat_currency: String,
    withdraw: &Option<WithdrawTransaction>,
) -> FiatTransactionUpdate {
    let withdraw_status = withdraw.as_ref().map(|w| w.status.as_str());
    let status = map_status(status, withdraw_status);

    FiatTransactionUpdate {
        transaction_id: merchant_transaction_id,
        provider_transaction_id: None,
        status,
        transaction_hash: withdraw.as_ref().and_then(|w| w.hash.clone()),
        fiat_amount: Some(fiat_amount),
        fiat_currency: Some(fiat_currency.to_ascii_uppercase()),
    }
}

fn map_buy_transaction(buy: BuyTransaction, withdraw: Option<WithdrawTransaction>) -> FiatTransactionUpdate {
    map_buy_like_transaction(&buy.status, buy.merchant_transaction_id, buy.fiat_amount, buy.fiat_currency, &withdraw)
}

fn map_buy_acquirer_transaction(buy_acquirer: BuyAcquirerTransaction, withdraw: Option<WithdrawTransaction>) -> FiatTransactionUpdate {
    map_buy_like_transaction(
        &buy_acquirer.status,
        buy_acquirer.merchant_transaction_id,
        buy_acquirer.fiat_amount,
        buy_acquirer.fiat_currency,
        &withdraw,
    )
}

fn map_mobile_pay_transaction(mobile_pay: MobilePayTransaction, withdraw: Option<WithdrawTransaction>) -> FiatTransactionUpdate {
    map_buy_like_transaction(
        &mobile_pay.status,
        mobile_pay.merchant_transaction_id,
        mobile_pay.fiat_amount,
        mobile_pay.fiat_currency,
        &withdraw,
    )
}

fn map_sell_transaction(withdraw: WithdrawTransaction) -> FiatTransactionUpdate {
    let status = map_status(&withdraw.status, None);

    FiatTransactionUpdate {
        transaction_id: withdraw.merchant_transaction_id,
        provider_transaction_id: None,
        status,
        transaction_hash: withdraw.hash,
        fiat_amount: Some(withdraw.amount),
        fiat_currency: None,
    }
}

fn map_sell_transaction_new(sell: SellTransaction, deposit: Option<DepositTransaction>) -> FiatTransactionUpdate {
    let status = map_status(&sell.status, None);

    FiatTransactionUpdate {
        transaction_id: sell.merchant_transaction_id,
        provider_transaction_id: None,
        status,
        transaction_hash: deposit.as_ref().and_then(|d| d.hash.clone()),
        fiat_amount: Some(sell.fiat_amount),
        fiat_currency: Some(sell.fiat_currency.to_ascii_uppercase()),
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
    use crate::providers::mercuryo::models::{Currencies, MercuryoTransactionResponse, Response, Webhook};
    use primitives::FiatTransactionStatus;

    #[test]
    fn test_map_order_from_buy_complete_response() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let response: Response<Vec<MercuryoTransactionResponse>> = serde_json::from_str(include_str!("../../../testdata/mercuryo/transaction_buy_complete.json"))?;
        let transaction = response.data.into_iter().next().unwrap();

        let result = map_order_from_response(transaction)?;

        assert_eq!(result.transaction_id, "f2e68ddb-ee2b-42ba");
        assert_eq!(result.provider_transaction_id, None);
        assert_eq!(result.fiat_amount, Some(99.0));
        assert_eq!(result.fiat_currency, Some("USD".to_string()));
        assert!(matches!(result.status, FiatTransactionStatus::Complete));

        Ok(())
    }

    #[test]
    fn test_map_order_from_mobile_buy_complete_response() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let response: Response<Vec<MercuryoTransactionResponse>> = serde_json::from_str(include_str!("../../../testdata/mercuryo/transaction_buy_mobile_complete.json"))?;
        let transaction = response.data.into_iter().next().unwrap();

        let result = map_order_from_response(transaction)?;

        assert_eq!(result.transaction_id, "036fecc9-0b73-4875-bd6f-2b868e30cf55");
        assert_eq!(result.provider_transaction_id, None);
        assert_eq!(result.fiat_amount, Some(20.0));
        assert_eq!(result.fiat_currency, Some("USD".to_string()));
        assert!(matches!(result.status, FiatTransactionStatus::Complete));
        assert_eq!(result.transaction_hash, Some("0x".to_string()));

        Ok(())
    }

    #[test]
    fn test_map_order_from_sell_complete_response() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let response: Response<Vec<MercuryoTransactionResponse>> = serde_json::from_str(include_str!("../../../testdata/mercuryo/transaction_sell_complete.json"))?;
        let transaction = response.data.into_iter().next().unwrap();

        let result = map_order_from_response(transaction)?;

        assert_eq!(result.transaction_id, "d9c80819-e0b2-4f6e-8a59-3eb6321daa4e");
        assert_eq!(result.provider_transaction_id, None);
        assert_eq!(result.fiat_amount, Some(29.32));
        assert_eq!(result.fiat_currency, Some("USD".to_string()));
        assert!(matches!(result.status, FiatTransactionStatus::Complete));
        assert_eq!(
            result.transaction_hash,
            Some("0xf07ff7195ac58ac23999aac7089210718f3f528167f05ba7880c5850648f66d4".to_string())
        );

        Ok(())
    }

    #[test]
    fn test_map_order_from_buy_acquirer_complete_response() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let response: Response<Vec<MercuryoTransactionResponse>> = serde_json::from_str(include_str!("../../../testdata/mercuryo/transaction_buy_acquirer_complete.json"))?;
        let transaction = response.data.into_iter().next().unwrap();

        let result = map_order_from_response(transaction)?;

        assert_eq!(result.transaction_id, "00000000-0000-0000-0000-000000000001");
        assert_eq!(result.provider_transaction_id, None);
        assert_eq!(result.fiat_amount, Some(123.0));
        assert_eq!(result.fiat_currency, Some("EUR".to_string()));
        assert!(matches!(result.status, FiatTransactionStatus::Complete));
        assert_eq!(
            result.transaction_hash,
            Some("0x0000000000000000000000000000000000000000000000000000000000000001".to_string())
        );

        Ok(())
    }

    #[test]
    fn test_map_order_from_webhook() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let webhook: Webhook = serde_json::from_str(include_str!("../../../testdata/mercuryo/webhook_buy_complete.json"))?;

        let result = map_order_from_webhook(webhook.data);

        assert_eq!(result.transaction_id, "f34d79d6-b8d4-4213-a29d-756467d003cb");
        assert_eq!(result.provider_transaction_id, None);
        assert_eq!(result.status, FiatTransactionStatus::Failed);
        assert_eq!(result.fiat_amount, Some(270.0));
        assert_eq!(result.fiat_currency, Some("USD".to_string()));
        assert_eq!(result.transaction_hash, None);

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
