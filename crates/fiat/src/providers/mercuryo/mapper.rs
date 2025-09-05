use super::{
    client::MercuryoClient,
    models::{Asset, BuyTransaction, DepositTransaction, MercuryoTransactionResponse, MobilePayTransaction, SellTransaction, WithdrawTransaction},
};
use crate::model::FiatProviderAsset;
use primitives::{AssetId, Chain, FiatQuoteType, FiatTransaction, FiatTransactionStatus};

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

fn map_buy_transaction(buy: BuyTransaction, withdraw: Option<WithdrawTransaction>) -> Result<FiatTransaction, Box<dyn std::error::Error + Send + Sync>> {
    let chain = map_symbol_to_chain(&buy.currency);
    let asset_id = chain.map(AssetId::from_chain);

    let status = match buy.status.as_str() {
        "new" | "pending" | "order_scheduled" => FiatTransactionStatus::Pending,
        "cancelled" | "order_failed" | "descriptor_failed" => FiatTransactionStatus::Failed,
        "paid" => {
            if let Some(ref w) = withdraw {
                match w.status.as_str() {
                    "completed" => FiatTransactionStatus::Complete,
                    "cancelled" | "failed" => FiatTransactionStatus::Failed,
                    _ => FiatTransactionStatus::Pending,
                }
            } else {
                FiatTransactionStatus::Pending
            }
        }
        "completed" | "succeeded" => FiatTransactionStatus::Complete,
        _ => FiatTransactionStatus::Unknown(buy.status.clone()),
    };

    Ok(FiatTransaction {
        asset_id,
        transaction_type: FiatQuoteType::Buy,
        symbol: buy.currency,
        provider_id: MercuryoClient::NAME.id(),
        provider_transaction_id: buy.merchant_transaction_id,
        status,
        country: buy.card_country.map(|c| c.to_uppercase()),
        fiat_amount: buy.fiat_amount,
        fiat_currency: buy.fiat_currency,
        transaction_hash: withdraw.as_ref().and_then(|w| w.hash.clone()),
        address: withdraw.as_ref().and_then(|w| w.address.clone()),
    })
}

fn map_mobile_pay_transaction(
    mobile_pay: MobilePayTransaction,
    withdraw: Option<WithdrawTransaction>,
) -> Result<FiatTransaction, Box<dyn std::error::Error + Send + Sync>> {
    let chain = map_symbol_to_chain(&mobile_pay.currency);
    let asset_id = chain.map(AssetId::from_chain);

    let status = match mobile_pay.status.as_str() {
        "new" | "pending" | "order_scheduled" => FiatTransactionStatus::Pending,
        "cancelled" | "order_failed" | "descriptor_failed" => FiatTransactionStatus::Failed,
        "paid" => {
            if let Some(ref w) = withdraw {
                match w.status.as_str() {
                    "completed" => FiatTransactionStatus::Complete,
                    "cancelled" | "failed" => FiatTransactionStatus::Failed,
                    _ => FiatTransactionStatus::Pending,
                }
            } else {
                FiatTransactionStatus::Pending
            }
        }
        "completed" | "succeeded" => FiatTransactionStatus::Complete,
        _ => FiatTransactionStatus::Unknown(mobile_pay.status.clone()),
    };

    Ok(FiatTransaction {
        asset_id,
        transaction_type: FiatQuoteType::Buy,
        symbol: mobile_pay.currency,
        provider_id: MercuryoClient::NAME.id(),
        provider_transaction_id: mobile_pay.merchant_transaction_id,
        status,
        country: mobile_pay.card_country.map(|c| c.to_uppercase()),
        fiat_amount: mobile_pay.fiat_amount,
        fiat_currency: mobile_pay.fiat_currency,
        transaction_hash: withdraw.as_ref().and_then(|w| w.hash.clone()),
        address: withdraw.as_ref().and_then(|w| w.address.clone()),
    })
}

fn map_sell_transaction(withdraw: WithdrawTransaction) -> Result<FiatTransaction, Box<dyn std::error::Error + Send + Sync>> {
    let status = match withdraw.status.as_str() {
        "new" | "pending" | "order_scheduled" => FiatTransactionStatus::Pending,
        "cancelled" | "order_failed" | "descriptor_failed" => FiatTransactionStatus::Failed,
        "paid" | "completed" | "succeeded" => FiatTransactionStatus::Complete,
        _ => FiatTransactionStatus::Unknown(withdraw.status.clone()),
    };

    let chain = map_symbol_to_chain(&withdraw.currency);
    let asset_id = chain.map(AssetId::from_chain);

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

    let status = match sell.status.as_str() {
        "new" | "pending" | "order_scheduled" => FiatTransactionStatus::Pending,
        "cancelled" | "order_failed" | "descriptor_failed" => FiatTransactionStatus::Failed,
        "paid" | "completed" | "succeeded" => FiatTransactionStatus::Complete,
        _ => FiatTransactionStatus::Unknown(sell.status.clone()),
    };

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

pub fn map_asset(asset: Asset) -> Option<FiatProviderAsset> {
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
        buy_limits: vec![],
        sell_limits: vec![],
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::mercuryo::models::{MercuryoTransactionResponse, Response};
    use primitives::{FiatQuoteType, FiatTransactionStatus};

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
}
