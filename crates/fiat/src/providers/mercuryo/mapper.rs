use super::{
    client::MercuryoClient,
    models::{
        Asset, BuyTransaction, DepositTransaction, FiatPaymentMethod, MercuryoTransactionResponse, MobilePayTransaction, SellTransaction, WithdrawTransaction,
    },
};
use crate::model::FiatProviderAsset;
use primitives::currency::Currency;
use primitives::fiat_assets::FiatAssetLimits;
use primitives::PaymentType;
use primitives::{AssetId, Chain, FiatQuoteType, FiatTransaction, FiatTransactionStatus};
use std::collections::HashMap;

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
        _ => None,
    }
}

pub fn map_symbol_to_chain(symbol: &str) -> Option<Chain> {
    match symbol.to_uppercase().as_str() {
        "BTC" => Some(Chain::Bitcoin),
        "BCH" => Some(Chain::BitcoinCash),
        "ETH" => Some(Chain::Ethereum),
        "BNB" => Some(Chain::SmartChain),
        "MATIC" => Some(Chain::Polygon),
        "SOL" => Some(Chain::Solana),
        "AVAX" => Some(Chain::AvalancheC),
        "TRX" => Some(Chain::Tron),
        "XRP" => Some(Chain::Xrp),
        "LTC" => Some(Chain::Litecoin),
        "DOGE" => Some(Chain::Doge),
        "DOT" => Some(Chain::Polkadot),
        "ADA" => Some(Chain::Cardano),
        "XLM" => Some(Chain::Stellar),
        "ALGO" => Some(Chain::Algorand),
        "NEAR" => Some(Chain::Near),
        "ATOM" => Some(Chain::Cosmos),
        "FTM" => Some(Chain::Fantom),
        "CELO" => Some(Chain::Celo),
        "TON" => Some(Chain::Ton),
        "SUI" => Some(Chain::Sui),
        "APT" => Some(Chain::Aptos),
        _ => None, // For ERC-20 tokens and others, default to Ethereum
    }
}

pub fn map_order_from_response(transaction: MercuryoTransactionResponse) -> Result<FiatTransaction, Box<dyn std::error::Error + Send + Sync>> {
    if let Some(buy) = transaction.buy {
        return map_buy_transaction(buy, transaction.withdraw);
    }

    if let Some(mobile_pay) = transaction.mobile_pay {
        return map_mobile_pay_transaction(mobile_pay, transaction.withdraw);
    }

    if let Some(sell) = transaction.sell {
        return map_sell_transaction_new(sell, transaction.deposit);
    }

    if let Some(withdraw) = transaction.withdraw {
        return map_sell_transaction(withdraw);
    }

    Err("No valid transaction data found".into())
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
        _ => FiatTransactionStatus::Unknown(status.to_string()),
    }
}

fn map_buy_like_transaction(
    currency: &str,
    status: &str,
    merchant_transaction_id: String,
    card_country: Option<String>,
    fiat_amount: f64,
    fiat_currency: String,
    withdraw: &Option<WithdrawTransaction>,
) -> Result<FiatTransaction, Box<dyn std::error::Error + Send + Sync>> {
    let chain = map_symbol_to_chain(currency);
    let asset_id = chain.map(AssetId::from_chain);
    let withdraw_status = withdraw.as_ref().map(|w| w.status.as_str());
    let status = map_status(status, withdraw_status);

    Ok(FiatTransaction {
        asset_id,
        transaction_type: FiatQuoteType::Buy,
        symbol: currency.to_string(),
        provider_id: MercuryoClient::NAME.id(),
        provider_transaction_id: merchant_transaction_id,
        status,
        country: card_country.map(|c| c.to_uppercase()),
        fiat_amount,
        fiat_currency,
        transaction_hash: withdraw.as_ref().and_then(|w| w.hash.clone()),
        address: withdraw.as_ref().and_then(|w| w.address.clone()),
    })
}

fn map_buy_transaction(buy: BuyTransaction, withdraw: Option<WithdrawTransaction>) -> Result<FiatTransaction, Box<dyn std::error::Error + Send + Sync>> {
    map_buy_like_transaction(
        &buy.currency,
        &buy.status,
        buy.merchant_transaction_id,
        buy.card_country,
        buy.fiat_amount,
        buy.fiat_currency,
        &withdraw,
    )
}

fn map_mobile_pay_transaction(
    mobile_pay: MobilePayTransaction,
    withdraw: Option<WithdrawTransaction>,
) -> Result<FiatTransaction, Box<dyn std::error::Error + Send + Sync>> {
    map_buy_like_transaction(
        &mobile_pay.currency,
        &mobile_pay.status,
        mobile_pay.merchant_transaction_id,
        mobile_pay.card_country,
        mobile_pay.fiat_amount,
        mobile_pay.fiat_currency,
        &withdraw,
    )
}

fn map_sell_transaction(withdraw: WithdrawTransaction) -> Result<FiatTransaction, Box<dyn std::error::Error + Send + Sync>> {
    let chain = map_symbol_to_chain(&withdraw.currency);
    let asset_id = chain.map(AssetId::from_chain);
    let status = map_status(&withdraw.status, None);

    Ok(FiatTransaction {
        asset_id,
        transaction_type: FiatQuoteType::Sell,
        symbol: withdraw.currency,
        provider_id: MercuryoClient::NAME.id(),
        provider_transaction_id: withdraw.merchant_transaction_id,
        status,
        country: None,
        fiat_amount: withdraw.amount,
        fiat_currency: "USD".to_string(),
        transaction_hash: withdraw.hash,
        address: withdraw.address,
    })
}

fn map_sell_transaction_new(sell: SellTransaction, deposit: Option<DepositTransaction>) -> Result<FiatTransaction, Box<dyn std::error::Error + Send + Sync>> {
    let chain = map_symbol_to_chain(&sell.currency);
    let asset_id = chain.map(AssetId::from_chain);
    let status = map_status(&sell.status, None);

    Ok(FiatTransaction {
        asset_id,
        transaction_type: FiatQuoteType::Sell,
        symbol: sell.currency,
        provider_id: MercuryoClient::NAME.id(),
        provider_transaction_id: sell.merchant_transaction_id,
        status,
        country: None,
        fiat_amount: sell.fiat_amount,
        fiat_currency: sell.fiat_currency,
        transaction_hash: deposit.as_ref().and_then(|d| d.hash.clone()),
        address: deposit.as_ref().and_then(|d| d.address.clone()),
    })
}

fn map_asset_base(asset: Asset, buy_limits: Vec<FiatAssetLimits>, sell_limits: Vec<FiatAssetLimits>) -> Option<FiatProviderAsset> {
    let chain = map_asset_chain(asset.network.clone());
    let token_id = if asset.contract.is_empty() { None } else { Some(asset.contract.clone()) };
    Some(FiatProviderAsset {
        id: asset.clone().currency + "_" + asset.network.as_str(),
        chain,
        token_id,
        symbol: asset.clone().currency,
        network: Some(asset.network),
        enabled: true,
        unsupported_countries: None,
        buy_limits,
        sell_limits,
    })
}

pub fn map_asset(asset: Asset) -> Option<FiatProviderAsset> {
    map_asset_base(asset, vec![], vec![])
}

pub fn map_asset_with_limits(asset: Asset, fiat_payment_methods: &HashMap<String, FiatPaymentMethod>) -> Option<FiatProviderAsset> {
    let buy_limits = map_limits(fiat_payment_methods, FiatQuoteType::Buy);
    let sell_limits = map_limits(fiat_payment_methods, FiatQuoteType::Sell);
    map_asset_base(asset, buy_limits, sell_limits)
}

fn map_limits(fiat_payment_methods: &HashMap<String, FiatPaymentMethod>, quote_type: FiatQuoteType) -> Vec<FiatAssetLimits> {
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
                        quote_type: quote_type.clone(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::mercuryo::models::{Currencies, MercuryoTransactionResponse, Response};
    use primitives::currency::Currency;
    use primitives::{FiatQuoteType, FiatTransactionStatus, PaymentType};

    #[tokio::test]
    async fn test_map_order_from_buy_complete_response() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let response: Response<Vec<MercuryoTransactionResponse>> =
            serde_json::from_str(include_str!("../../../testdata/mercuryo/transaction_buy_complete.json"))?;
        let transaction = response.data.into_iter().next().unwrap();

        let result = map_order_from_response(transaction)?;

        assert_eq!(result.provider_transaction_id, "f2e68ddb-ee2b-42ba");
        assert!(matches!(result.transaction_type, FiatQuoteType::Buy));
        assert_eq!(result.symbol, "ETH");
        assert_eq!(result.fiat_currency, "USD");
        assert_eq!(result.fiat_amount, 99.0);
        assert!(matches!(result.status, FiatTransactionStatus::Complete));
        assert_eq!(result.country, Some("US".to_string()));

        Ok(())
    }

    #[tokio::test]
    async fn test_map_order_from_mobile_buy_complete_response() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let response: Response<Vec<MercuryoTransactionResponse>> =
            serde_json::from_str(include_str!("../../../testdata/mercuryo/transaction_buy_mobile_complete.json"))?;
        let transaction = response.data.into_iter().next().unwrap();

        let result = map_order_from_response(transaction)?;

        assert_eq!(result.provider_transaction_id, "036fecc9-0b73-4875-bd6f-2b868e30cf55");
        assert!(matches!(result.transaction_type, FiatQuoteType::Buy));
        assert_eq!(result.symbol, "TON");
        assert_eq!(result.fiat_currency, "USD");
        assert_eq!(result.fiat_amount, 20.0);
        assert!(matches!(result.status, FiatTransactionStatus::Complete));
        assert_eq!(result.country, None);
        assert_eq!(result.transaction_hash, Some("0x".to_string()));
        assert_eq!(result.address, Some("UQByrYTVrJMttx88DbubyjV4nKy3waokfenb4QGJOP55Nn99".to_string()));

        Ok(())
    }

    #[tokio::test]
    async fn test_map_order_from_sell_complete_response() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let response: Response<Vec<MercuryoTransactionResponse>> =
            serde_json::from_str(include_str!("../../../testdata/mercuryo/transaction_sell_complete.json"))?;
        let transaction = response.data.into_iter().next().unwrap();

        let result = map_order_from_response(transaction)?;

        assert_eq!(result.provider_transaction_id, "d9c80819-e0b2-4f6e-8a59-3eb6321daa4e");
        assert!(matches!(result.transaction_type, FiatQuoteType::Sell));
        assert_eq!(result.symbol, "ETH");
        assert_eq!(result.fiat_currency, "USD");
        assert_eq!(result.fiat_amount, 29.32);
        assert!(matches!(result.status, FiatTransactionStatus::Complete));
        assert_eq!(result.country, None);
        assert_eq!(
            result.transaction_hash,
            Some("0xf07ff7195ac58ac23999aac7089210718f3f528167f05ba7880c5850648f66d4".to_string())
        );
        assert_eq!(result.address, Some("0x9837C4e9A7afac29eBd170b1b68AE16F3dE38298".to_string()));

        Ok(())
    }

    #[tokio::test]
    async fn test_map_asset_with_limits() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let currencies = serde_json::from_str::<Response<Currencies>>(include_str!("../../../testdata/mercuryo/assets.json"))?.data;
        let asset = currencies.config.crypto_currencies.first().unwrap();

        let result = map_asset_with_limits(asset.clone(), &currencies.fiat_payment_methods).unwrap();

        assert!(!result.buy_limits.is_empty());
        assert!(!result.sell_limits.is_empty());

        let usd_buy_limit = result
            .buy_limits
            .iter()
            .find(|limit| limit.currency == Currency::USD && limit.payment_type == PaymentType::Card);
        assert!(usd_buy_limit.is_some());
        assert_eq!(usd_buy_limit.unwrap().min_amount, Some(29.33));
        assert_eq!(usd_buy_limit.unwrap().max_amount, Some(5865.20));

        Ok(())
    }

    #[tokio::test]
    async fn test_map_trump_asset() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let currencies = serde_json::from_str::<Response<Currencies>>(include_str!("../../../testdata/mercuryo/assets.json"))?.data;
        
        let trump_asset = currencies.config.crypto_currencies
            .iter()
            .find(|asset| asset.currency == "TRUMP" && asset.network == "SOLANA")
            .unwrap();

        let result = map_asset_with_limits(trump_asset.clone(), &currencies.fiat_payment_methods).unwrap();

        assert_eq!(result.symbol, "TRUMP");
        assert_eq!(result.chain, Some(Chain::Solana));
        assert_eq!(result.token_id, Some("6p6xgHyF7AeE6TZkSmFsko444wqoP15icUSqi2jfGiPN".to_string()));
        assert_eq!(result.network, Some("SOLANA".to_string()));
        assert_eq!(result.id, "TRUMP_SOLANA");

        Ok(())
    }

    #[tokio::test]
    async fn test_contract_assets_mapping() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let currencies = serde_json::from_str::<Response<Currencies>>(include_str!("../../../testdata/mercuryo/assets.json"))?.data;
        
        let all_assets: Vec<_> = currencies.config.crypto_currencies
            .into_iter()
            .flat_map(|asset| map_asset_with_limits(asset, &currencies.fiat_payment_methods))
            .collect();

        let contract_assets: Vec<_> = all_assets
            .iter()
            .filter(|asset| asset.token_id.is_some())
            .collect();

        let trump_assets: Vec<_> = all_assets
            .iter()
            .filter(|asset| asset.symbol == "TRUMP")
            .collect();

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
}
