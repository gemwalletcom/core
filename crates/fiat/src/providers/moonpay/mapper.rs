use crate::providers::moonpay::models::{Asset, FiatCurrencyType, Transaction};
use primitives::{Chain, FiatQuoteType, FiatTransactionStatus, FiatTransactionUpdate};

#[cfg(test)]
use primitives::PaymentType;
pub fn map_asset_chain(asset: Asset) -> Option<Chain> {
    match asset.metadata?.network_code.as_str() {
        "ethereum" => Some(Chain::Ethereum),
        "binance_smart_chain" | "bnb_chain" => Some(Chain::SmartChain),
        "solana" => Some(Chain::Solana),
        "arbitrum" => Some(Chain::Arbitrum),
        "base" => Some(Chain::Base),
        "avalanche_c_chain" => Some(Chain::AvalancheC),
        "optimism" => Some(Chain::Optimism),
        "polygon" => Some(Chain::Polygon),
        "tron" => Some(Chain::Tron),
        "aptos" => Some(Chain::Aptos),
        "bitcoin" => Some(Chain::Bitcoin),
        "bitcoin_cash" => Some(Chain::BitcoinCash),
        "dogecoin" => Some(Chain::Doge),
        "litecoin" => Some(Chain::Litecoin),
        "ripple" => Some(Chain::Xrp),
        "sui" => Some(Chain::Sui),
        "ton" => Some(Chain::Ton),
        "cosmos" => Some(Chain::Cosmos),
        "near" => Some(Chain::Near),
        "linea" => Some(Chain::Linea),
        "zksync" => Some(Chain::ZkSync),
        "celo" => Some(Chain::Celo),
        "stellar" => Some(Chain::Stellar),
        "algorand" => Some(Chain::Algorand),
        "polkadot" => Some(Chain::Polkadot),
        "berachain" => Some(Chain::Berachain),
        "sonic" => Some(Chain::Sonic),
        "celestia" => Some(Chain::Celestia),
        "noble" => Some(Chain::Noble),
        "worldchain" => Some(Chain::World),
        "injective" => Some(Chain::Injective),
        "cardano" => Some(Chain::Cardano),
        "monad" => Some(Chain::Monad),
        _ => None,
    }
}

pub fn map_order(payload: Transaction) -> FiatTransactionUpdate {
    let transaction_id = payload.external_transaction_id.clone().unwrap_or_else(|| payload.id.clone());
    let provider_transaction_id = (transaction_id != payload.id).then_some(payload.id.clone());
    let transaction_type = if payload.base_currency.currency_type == FiatCurrencyType::Fiat {
        FiatQuoteType::Buy
    } else {
        FiatQuoteType::Sell
    };
    let currency_amount = match transaction_type {
        FiatQuoteType::Buy => payload.base_currency_amount.unwrap_or_default(),
        FiatQuoteType::Sell => payload.quote_currency_amount.unwrap_or_default(),
    };
    let status = map_status(&payload.status);
    let fee_provider = payload.fee_amount.unwrap_or_default();
    let fee_network = payload.network_fee_amount.unwrap_or_default();
    let fee_partner = payload.extra_fee_amount.unwrap_or_default();

    let fiat_amount = match transaction_type {
        FiatQuoteType::Buy => currency_amount + fee_provider + fee_network + fee_partner,
        FiatQuoteType::Sell => currency_amount,
    };

    let address = match transaction_type {
        FiatQuoteType::Buy => payload.wallet_address,
        FiatQuoteType::Sell => payload.refund_wallet_address.or(payload.wallet_address),
    };

    FiatTransactionUpdate {
        transaction_id,
        provider_transaction_id,
        status,
        transaction_hash: payload.crypto_transaction_id,
        address,
        fiat_amount: Some(fiat_amount),
    }
}

fn map_status(status: &str) -> FiatTransactionStatus {
    match status {
        "pending" | "waitingForDeposit" => FiatTransactionStatus::Pending,
        "failed" => FiatTransactionStatus::Failed,
        "completed" => FiatTransactionStatus::Complete,
        _ if status.starts_with("waiting") => FiatTransactionStatus::Pending,
        _ => FiatTransactionStatus::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::moonpay::client::MoonPayClient;
    use crate::providers::moonpay::models::{Data, Transaction};

    #[test]
    fn test_map_order_buy_failed() {
        let webhook_data: Data<Transaction> = serde_json::from_str(include_str!("../../../testdata/moonpay/webhook_buy_complete.json")).expect("Failed to parse test data");
        let payload = webhook_data.data;

        let result = map_order(payload);

        assert_eq!(result.transaction_id, "1b6cdb1e-9299-45b1-9670-54db1ea5a21f");
        assert_eq!(result.provider_transaction_id, None);
        assert!(matches!(result.status, FiatTransactionStatus::Failed));
        assert_eq!(result.fiat_amount, Some(20.0)); // 15.39 + 3.99 + 0.47 + 0.15
        assert_eq!(result.address, Some("TYxT3F8pdkTDkhw4JsfodKnEgaYpNaANmW".to_string()));
    }

    #[test]
    fn test_map_order_sell_pending() {
        let webhook_data: Data<Transaction> = serde_json::from_str(include_str!("../../../testdata/moonpay/webhook_sell_complete_.json")).expect("Failed to parse test data");
        let payload = webhook_data.data;

        let result = map_order(payload);

        assert_eq!(result.transaction_id, "557d8fc1-0657-4505-8702-6bd9e1cd6241");
        assert_eq!(result.provider_transaction_id, None);
        assert!(matches!(result.status, FiatTransactionStatus::Pending));
        assert_eq!(result.fiat_amount, Some(3123.07)); // quoteCurrencyAmount - what user actually receives
        assert_eq!(result.address, Some("0xd41fdb03ba84762dd66a0af1a6c8540ff1ba5dfb".to_string()));
    }

    #[test]
    fn test_map_order_v3_sell_complete() {
        let webhook_data: Transaction = serde_json::from_str(include_str!("../../../testdata/moonpay/sell_transaction_complete.json")).expect("Failed to parse test data");

        let result = map_order(webhook_data);

        assert_eq!(result.transaction_id, "bcd0315e-4264-48bb-8c10-1a5207297341");
        assert_eq!(result.provider_transaction_id, None);
        assert!(matches!(result.status, FiatTransactionStatus::Complete));
        assert_eq!(result.fiat_amount, Some(3123.07)); // quoteCurrencyAmount - what user actually receives
        assert_eq!(result.address, Some("0xd41fdb03ba84762dd66a0af1a6c8540ff1ba5dfb".to_string()));
        assert_eq!(result.transaction_hash, Some("0xabc123456789".to_string()));
    }

    #[test]
    fn test_map_order_sell_failed() {
        let webhook_data: Transaction = serde_json::from_str(include_str!("../../../testdata/moonpay/transaction_sell_failed.json")).expect("Failed to parse test data");

        let result = map_order(webhook_data);

        assert_eq!(result.transaction_id, "bcd0315e-4264-48bb-8c10-1a5207297341");
        assert_eq!(result.provider_transaction_id, None);
        assert!(matches!(result.status, FiatTransactionStatus::Failed));
        assert_eq!(result.fiat_amount, Some(8419.77)); // quoteCurrencyAmount - what user actually receives
        assert_eq!(result.address, Some("qpm2qsznhks23z7629mms6s4cwef74vcwvy22gdx6a".to_string()));
    }

    #[test]
    fn test_map_order_buy_waiting_payment() {
        let payload: Transaction = serde_json::from_value(serde_json::json!({
            "id": "9a1a7efe-c6f1-4c69-ad9f-6abd2a7c6385",
            "externalTransactionId": null,
            "status": "waitingPayment",
            "baseCurrencyAmount": 66.53,
            "quoteCurrencyAmount": null,
            "baseCurrency": {
                "code": "eur",
                "metadata": null,
                "type": "fiat",
                "isSuspended": false,
                "isBaseAsset": false,
                "isSellSupported": true,
                "notAllowedCountries": [],
                "minBuyAmount": 20.0,
                "maxBuyAmount": 30000.0,
                "minSellAmount": null,
                "maxSellAmount": null
            },
            "currency": null,
            "quoteCurrency": null,
            "walletAddress": "bc1qacjrh89lukj82agg2ujze5l2e3nss3pru96733",
            "refundWalletAddress": null,
            "cryptoTransactionId": null,
            "networkFeeAmount": 0.81,
            "extraFeeAmount": 0.67,
            "feeAmount": 3.99,
            "country": "ITA",
            "failureReason": null
        }))
        .expect("Failed to parse test payload");

        let result = map_order(payload);

        assert_eq!(result.transaction_id, "9a1a7efe-c6f1-4c69-ad9f-6abd2a7c6385");
        assert_eq!(result.provider_transaction_id, None);
        assert_eq!(result.status, FiatTransactionStatus::Pending);
        assert_eq!(result.fiat_amount, Some(72.0));
        assert_eq!(result.address, Some("bc1qacjrh89lukj82agg2ujze5l2e3nss3pru96733".to_string()));
    }

    #[test]
    fn test_map_asset_with_limits() {
        let assets: Vec<Asset> = serde_json::from_str(include_str!("../../../testdata/moonpay/assets.json")).expect("Failed to parse test data");
        let cardano = assets.iter().find(|a| a.code == "ada").expect("Cardano not found");

        let result = MoonPayClient::map_asset(cardano.clone()).expect("Failed to map asset");

        assert_eq!(result.symbol, "ada");
        assert_eq!(result.chain, Some(Chain::Cardano));
        assert!(result.enabled);

        assert_eq!(result.buy_limits.len(), 3);
        let card_limit = result.buy_limits.iter().find(|limit| limit.payment_type == PaymentType::Card).unwrap();
        assert_eq!(card_limit.min_amount, Some(6.1));
        assert_eq!(card_limit.max_amount, None);

        assert!(result.buy_limits.iter().any(|limit| limit.payment_type == PaymentType::ApplePay));
        assert!(result.buy_limits.iter().any(|limit| limit.payment_type == PaymentType::GooglePay));

        assert_eq!(result.sell_limits.len(), 3);
        let sell_card_limit = result.sell_limits.iter().find(|limit| limit.payment_type == PaymentType::Card).unwrap();
        assert_eq!(sell_card_limit.min_amount, Some(24.3607));
        assert_eq!(sell_card_limit.max_amount, Some(12000.0));

        assert!(result.sell_limits.iter().any(|limit| limit.payment_type == PaymentType::ApplePay));
        assert!(result.sell_limits.iter().any(|limit| limit.payment_type == PaymentType::GooglePay));
    }

    #[test]
    fn test_skip_token_without_contract() {
        assert!(MoonPayClient::map_asset(Asset::mock("sweat_near", "near", None, false)).is_none());
        assert!(MoonPayClient::map_asset(Asset::mock("near", "near", None, true)).is_some());
    }
}
