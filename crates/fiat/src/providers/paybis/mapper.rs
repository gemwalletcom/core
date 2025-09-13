use std::str::FromStr;

use primitives::currency::Currency;
use primitives::{AssetId, Chain, FiatProviderName, FiatQuoteType, FiatTransaction, FiatTransactionStatus, PaymentType};
use streamer::FiatWebhook;

use crate::model::FiatProviderAsset;
use crate::providers::paybis::models::{PaybisData, PaymentMethodWithLimits};
use primitives::fiat_assets::FiatAssetLimits;

use super::{
    client::PaybisClient,
    models::{Currency as PaybisCurrency, PaybisWebhookData},
};

pub fn map_asset_id(currency: PaybisCurrency) -> Option<AssetId> {
    if !currency.is_crypto() {
        return None;
    }
    map_symbol_to_asset_id(&currency.code)
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
        "LTC" => Some(AssetId::from_chain(Chain::Litecoin)),
        "DOT" => Some(AssetId::from_chain(Chain::Polkadot)),
        "CELO" => Some(AssetId::from_chain(Chain::Celo)),
        "TON" => Some(AssetId::from_chain(Chain::Ton)),
        "DOGE" => Some(AssetId::from_chain(Chain::Doge)),

        "AVAXC" => Some(AssetId::from_chain(Chain::AvalancheC)),

        "ETH-BASE" => Some(AssetId::from_chain(Chain::Base)),
        "USDC-BASE" => Some(AssetId::from(Chain::Base, Some("0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913".to_string()))),

        "POL" => Some(AssetId::from_chain(Chain::Polygon)),
        "USDC-POLYGON" => Some(AssetId::from(Chain::Polygon, Some("0x3c499c542cEF5E3811e1192ce70d8cC03d5c3359".to_string()))),
        "USDT-POLYGON" => Some(AssetId::from(Chain::Polygon, Some("0xc2132D05D31c914a87C6611C10748AEb04B58e8F".to_string()))),

        "USDC-SOL" => Some(AssetId::from(Chain::Solana, Some("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string()))),
        "USDT-SOL" => Some(AssetId::from(Chain::Solana, Some("Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB".to_string()))),
        "BONK-SOL" => Some(AssetId::from(Chain::Solana, Some("DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263".to_string()))),

        "USDT-TRC20" => Some(AssetId::from(Chain::Tron, Some("TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t".to_string()))),

        "BNB" | "BNBSC" => Some(AssetId::from_chain(Chain::SmartChain)),
        "CAKE" => Some(AssetId::from(Chain::SmartChain, Some("0x0E09FaBB73Bd3Ade0a17ECC321fD13a19e81cE82".to_string()))),
        "TWT" => Some(AssetId::from(Chain::SmartChain, Some("0x4B0F1812e5Df2A09796481Ff14017e6005508003".to_string()))),

        "USDC" => Some(AssetId::from(Chain::Ethereum, Some("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string()))),
        "USDT" => Some(AssetId::from(Chain::Ethereum, Some("0xdAC17F958D2ee523a2206206994597C13D831ec7".to_string()))),
        "DAI" => Some(AssetId::from(Chain::Ethereum, Some("0x6B175474E89094C44Da98b954EedeAC495271d0F".to_string()))),

        "LINK" => Some(AssetId::from(Chain::Ethereum, Some("0x514910771AF9Ca656af840dff83E8264EcF986CA".to_string()))),
        "AAVE" => Some(AssetId::from(Chain::Ethereum, Some("0x7Fc66500c84A76Ad7e9c93437bFc5Ac33E2DDaE9".to_string()))),
        "UNI" => Some(AssetId::from(Chain::Ethereum, Some("0x1f9840a85d5aF5bf1D1762F925BDADdC4201F984".to_string()))),
        "MKR" => Some(AssetId::from(Chain::Ethereum, Some("0x9f8F72aA9304c8B593d555F12eF6589cC3A579A2".to_string()))),
        "COMP" => Some(AssetId::from(Chain::Ethereum, Some("0xc00e94Cb662C3520282E6f5717214004A7f26888".to_string()))),
        "CRV" => Some(AssetId::from(Chain::Ethereum, Some("0xD533a949740bb3306d119CC777fa900bA034cd52".to_string()))),
        "LDO" => Some(AssetId::from(Chain::Ethereum, Some("0x5A98FcBEA516Cf06857215779Fd812CA3beF1B32".to_string()))),
        "ENS" => Some(AssetId::from(Chain::Ethereum, Some("0xC18360217D8F7Ab5e7c516566761Ea12Ce7F9D72".to_string()))),
        "SUSHI" => Some(AssetId::from(Chain::Ethereum, Some("0x6B3595068778DD592e39A122f4f5a5cF09C90fE2".to_string()))),

        "SHIB" => Some(AssetId::from(Chain::Ethereum, Some("0x95aD61b0a150d79219dCF64E1E6Cc01f0B64C4cE".to_string()))),
        "PEPE" => Some(AssetId::from(Chain::Ethereum, Some("0x6982508145454Ce325dDbE47a25d4ec3d2311933".to_string()))),
        "APE" => Some(AssetId::from(Chain::Ethereum, Some("0x4d224452801ACEd8B2F0aebE155379bb5D594381".to_string()))),
        "SAND" => Some(AssetId::from(Chain::Ethereum, Some("0x3845badAde8e6dFF049820680d1F14bD3903a5d0".to_string()))),
        "BAT" => Some(AssetId::from(Chain::Ethereum, Some("0x0D8775F648430679A709E98d2b0Cb6250d2887EF".to_string()))),
        "FET" => Some(AssetId::from(Chain::Ethereum, Some("0xaea46A60368A7bD060eec7DF8CBa43b7EF41Ad85".to_string()))),
        "IMX" => Some(AssetId::from(Chain::Ethereum, Some("0xF57e7e7C23978C3cAEC3C3548E3D615c346e79fF".to_string()))),
        "CHZ" => Some(AssetId::from(Chain::Ethereum, Some("0x3506424F91fD33084466F402d5D97f05F8e3b4AF".to_string()))),
        "AXS" => Some(AssetId::from(Chain::Ethereum, Some("0xBB0E17EF65F82Ab018d8EDd776e8DD940327B28b".to_string()))),
        "DYDX" => Some(AssetId::from(Chain::Ethereum, Some("0x92D6C1e31e14520e676a687F0a93788B716BEff5".to_string()))),
        "ONEINCH" => Some(AssetId::from(Chain::Ethereum, Some("0x111111111117dC0aa78b770fA6A738034120C302".to_string()))),
        "GNO" => Some(AssetId::from(Chain::Ethereum, Some("0x6810e776880C02933D47DB1b9fc05908e5386b96".to_string()))),
        "QNT" => Some(AssetId::from(Chain::Ethereum, Some("0x4a220E6096B25EADb88358cb44068A3248254675".to_string()))),
        "NEXO" => Some(AssetId::from(Chain::Ethereum, Some("0xB62132e35a6c13ee1EE0f84dC5d40bad8d815206".to_string()))),
        "HOT" => Some(AssetId::from(Chain::Ethereum, Some("0x6c6EE5e31d828De241282B9606C8e98Ea48526E2".to_string()))),

        "ARB" => Some(AssetId::from(Chain::Arbitrum, Some("0x912CE59144191C1204E64559FE8253a0e49E6548".to_string()))),
        "OP" => Some(AssetId::from(Chain::Optimism, Some("0x4200000000000000000000000000000000000042".to_string()))),

        "USDC-STELLAR" => Some(AssetId::from(
            Chain::Stellar,
            Some("GA5ZSEJYB37JRC5AVCIA5MOP4RHTM335X2KGX3IHOJAPP5RE34K4KZVN::USDC".to_string()),
        )),

        _ => None,
    }
}

pub fn map_status(status: &str) -> FiatTransactionStatus {
    match status {
        "started" | "pending" | "confirming" | "payment-authorized" | "paid" => FiatTransactionStatus::Pending,
        "completed" | "success" => FiatTransactionStatus::Complete,
        "failed" | "cancelled" | "canceled" | "rejected" => FiatTransactionStatus::Failed,
        _ => FiatTransactionStatus::Unknown(status.to_string()),
    }
}

pub fn map_process_webhook(data: serde_json::Value) -> FiatWebhook {
    match serde_json::from_value::<PaybisData<PaybisWebhookData>>(data) {
        Ok(webhook) => map_webhook_data(webhook.data),
        Err(_) => FiatWebhook::None,
    }
}

pub fn map_webhook_data(webhook_data: PaybisWebhookData) -> FiatWebhook {
    FiatWebhook::Transaction(FiatTransaction {
        asset_id: map_symbol_to_asset_id(&webhook_data.amount_to.currency),
        transaction_type: match webhook_data.transaction.flow.as_str() {
            "buyCrypto" => FiatQuoteType::Buy,
            "sellCrypto" => FiatQuoteType::Sell,
            _ => FiatQuoteType::Buy,
        },
        symbol: webhook_data.amount_to.currency,
        provider_id: PaybisClient::NAME.id(),
        provider_transaction_id: webhook_data.transaction.invoice,
        status: map_status(&webhook_data.transaction.status),
        country: webhook_data
            .payment
            .as_ref()
            .and_then(|p| p.card.as_ref())
            .map(|c| c.billing_address.country.code.clone()),
        fiat_amount: webhook_data.amount_from.amount.parse().unwrap_or(0.0),
        fiat_currency: webhook_data.amount_from.currency.to_uppercase(),
        transaction_hash: webhook_data.payout.as_ref().and_then(|p| p.transaction_hash.clone()),
        address: webhook_data.payout.as_ref().and_then(|p| p.destination_wallet_address.clone()),
    })
}

fn map_asset(currency: PaybisCurrency, buy_limits: Vec<FiatAssetLimits>, sell_limits: Vec<FiatAssetLimits>) -> Option<FiatProviderAsset> {
    if !currency.is_crypto() {
        return None;
    }
    let asset = map_asset_id(currency.clone());
    Some(FiatProviderAsset {
        id: currency.code.clone(),
        provider: FiatProviderName::Paybis,
        chain: asset.as_ref().map(|x| x.chain),
        token_id: asset.as_ref().and_then(|x| x.token_id.clone()),
        symbol: currency.code.clone(),
        network: currency.blockchain_name.clone(),
        enabled: true,
        unsupported_countries: Some(currency.unsupported_countries()),
        buy_limits,
        sell_limits,
    })
}

pub fn map_assets(currencies: Vec<PaybisCurrency>) -> Vec<FiatProviderAsset> {
    currencies.into_iter().flat_map(|currency| map_asset(currency, vec![], vec![])).collect()
}

fn map_payment_type(payment_method_name: &str) -> Option<PaymentType> {
    match payment_method_name {
        "gem-wallet-credit-card" => Some(PaymentType::Card),
        "gem-wallet-google-pay-credit-card" => Some(PaymentType::GooglePay),
        "gem-wallet-apple-pay-credit-card" => Some(PaymentType::ApplePay),
        _ => None,
    }
}

pub fn map_assets_with_limits(currencies: Vec<PaybisCurrency>, limits: &PaybisData<Vec<PaymentMethodWithLimits>>) -> Vec<FiatProviderAsset> {
    currencies
        .into_iter()
        .filter_map(|currency| {
            let asset_buy_limits = limits
                .data
                .iter()
                .filter_map(|payment_method| map_payment_type(&payment_method.name).map(|payment_type| (payment_method, payment_type)))
                .flat_map(|(payment_method, payment_type)| {
                    payment_method.pairs.iter().filter_map({
                        let value = currency.code.clone();
                        move |currency_pair| {
                            currency_pair.to.iter().find(|c| c.currency_code == value).and_then(|currency_limit| {
                                Currency::from_str(currency_pair.from.as_str()).ok().map(|fiat_currency| FiatAssetLimits {
                                    currency: fiat_currency,
                                    payment_type: payment_type.clone(),
                                    min_amount: Some(currency_limit.min_amount),
                                    max_amount: Some(currency_limit.max_amount),
                                })
                            })
                        }
                    })
                })
                .collect();

            map_asset(currency, asset_buy_limits, vec![])
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_asset_id() {
        assert_eq!(
            map_asset_id(PaybisCurrency {
                code: "ETH".to_string(),
                blockchain_name: Some("ethereum".to_string()),
            }),
            Some(AssetId::from_chain(Chain::Ethereum))
        );

        assert_eq!(
            map_asset_id(PaybisCurrency {
                code: "BTC".to_string(),
                blockchain_name: Some("bitcoin".to_string()),
            }),
            Some(AssetId::from_chain(Chain::Bitcoin))
        );

        assert_eq!(
            map_asset_id(PaybisCurrency {
                code: "UNKNOWN".to_string(),
                blockchain_name: Some("unknown-chain".to_string()),
            }),
            None
        );

        assert_eq!(
            map_asset_id(PaybisCurrency {
                code: "USD".to_string(),
                blockchain_name: None,
            }),
            None
        );
    }

    #[test]
    fn test_map_symbol_to_asset_id_coins() {
        assert_eq!(map_symbol_to_asset_id("BTC"), Some(AssetId::from_chain(Chain::Bitcoin)));
        assert_eq!(map_symbol_to_asset_id("ETH"), Some(AssetId::from_chain(Chain::Ethereum)));
        assert_eq!(map_symbol_to_asset_id("TRX"), Some(AssetId::from_chain(Chain::Tron)));
        assert_eq!(map_symbol_to_asset_id("XRP"), Some(AssetId::from_chain(Chain::Xrp)));
        assert_eq!(map_symbol_to_asset_id("SOL"), Some(AssetId::from_chain(Chain::Solana)));
        assert_eq!(map_symbol_to_asset_id("ADA"), Some(AssetId::from_chain(Chain::Cardano)));
        assert_eq!(map_symbol_to_asset_id("DOT"), Some(AssetId::from_chain(Chain::Polkadot)));
        assert_eq!(map_symbol_to_asset_id("TON"), Some(AssetId::from_chain(Chain::Ton)));
        assert_eq!(map_symbol_to_asset_id("DOGE"), Some(AssetId::from_chain(Chain::Doge)));

        assert_eq!(
            map_symbol_to_asset_id("ARB"),
            Some(AssetId::from(Chain::Arbitrum, Some("0x912CE59144191C1204E64559FE8253a0e49E6548".to_string())))
        );
        assert_eq!(map_symbol_to_asset_id("AVAXC"), Some(AssetId::from_chain(Chain::AvalancheC)));
        assert_eq!(map_symbol_to_asset_id("POL"), Some(AssetId::from_chain(Chain::Polygon)));
        assert_eq!(map_symbol_to_asset_id("BNBSC"), Some(AssetId::from_chain(Chain::SmartChain)));

        assert_eq!(map_symbol_to_asset_id("ETH-BASE"), Some(AssetId::from_chain(Chain::Base)));

        assert_eq!(map_symbol_to_asset_id("UNKNOWN"), None);
    }

    #[test]
    fn test_map_symbol_to_asset_id_tokens() {
        let token_tests = vec![
            ("USDC", Chain::Ethereum, "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"),
            ("USDC-BASE", Chain::Base, "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913"),
            ("USDC-POLYGON", Chain::Polygon, "0x3c499c542cEF5E3811e1192ce70d8cC03d5c3359"),
            ("USDC-SOL", Chain::Solana, "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"),
            ("USDT", Chain::Ethereum, "0xdAC17F958D2ee523a2206206994597C13D831ec7"),
            ("USDT-POLYGON", Chain::Polygon, "0xc2132D05D31c914a87C6611C10748AEb04B58e8F"),
            ("USDT-SOL", Chain::Solana, "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB"),
            ("USDT-TRC20", Chain::Tron, "TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t"),
            ("LINK", Chain::Ethereum, "0x514910771AF9Ca656af840dff83E8264EcF986CA"),
            ("PEPE", Chain::Ethereum, "0x6982508145454Ce325dDbE47a25d4ec3d2311933"),
            ("MKR", Chain::Ethereum, "0x9f8F72aA9304c8B593d555F12eF6589cC3A579A2"),
            ("CRV", Chain::Ethereum, "0xD533a949740bb3306d119CC777fa900bA034cd52"),
            ("COMP", Chain::Ethereum, "0xc00e94Cb662C3520282E6f5717214004A7f26888"),
            ("CAKE", Chain::SmartChain, "0x0E09FaBB73Bd3Ade0a17ECC321fD13a19e81cE82"),
            ("BONK-SOL", Chain::Solana, "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263"),
        ];

        for (symbol, expected_chain, expected_token_id) in token_tests {
            let result = map_symbol_to_asset_id(symbol);
            let expected = Some(AssetId::from(expected_chain, Some(expected_token_id.to_string())));
            assert_eq!(result, expected, "Failed for symbol: {}", symbol);
        }
    }

    #[test]
    fn test_map_process_webhook() {
        let webhook_json: serde_json::Value = serde_json::from_str(include_str!("../../../testdata/paybis/webhook_transaction_started.json")).unwrap();

        let result = map_process_webhook(webhook_json);
        if let FiatWebhook::Transaction(transaction) = result {
            assert_eq!(transaction.provider_transaction_id, "PB21095868675TX1");
            assert_eq!(transaction.symbol, "SOL");
            assert_eq!(transaction.fiat_currency, "USD");
        } else {
            panic!("Expected FiatWebhook::Transaction variant");
        }
    }

    #[test]
    fn test_map_process_webhook_with_payment() {
        let webhook_json: serde_json::Value = serde_json::from_str(include_str!("../../../testdata/paybis/webhook_transaction_started.json")).unwrap();

        let result = map_process_webhook(webhook_json);
        if let FiatWebhook::Transaction(transaction) = result {
            assert_eq!(transaction.provider_transaction_id, "PB21095868675TX1");
            assert_eq!(transaction.symbol, "SOL");
            assert_eq!(transaction.fiat_currency, "USD");
            assert_eq!(transaction.fiat_amount, 50.0);
            assert!(matches!(transaction.transaction_type, FiatQuoteType::Buy));
            assert!(matches!(transaction.status, FiatTransactionStatus::Pending));
            assert_eq!(transaction.country, Some("US".to_string()));
            assert_eq!(transaction.address, Some("test123".to_string()));
            assert_eq!(transaction.transaction_hash, None);
        } else {
            panic!("Expected FiatWebhook::Transaction variant");
        }
    }

    #[test]
    fn test_map_process_webhook_no_payment() {
        let webhook_json: serde_json::Value =
            serde_json::from_str(include_str!("../../../testdata/paybis/webhook_transaction_started_no_payment.json")).unwrap();

        let result = map_process_webhook(webhook_json);
        if let FiatWebhook::Transaction(transaction) = result {
            assert_eq!(transaction.provider_transaction_id, "PB25095868675TX8");
            assert_eq!(transaction.symbol, "SOL");
            assert_eq!(transaction.fiat_currency, "USD");
            assert_eq!(transaction.country, None);
            assert_eq!(transaction.address, None);
        } else {
            panic!("Expected FiatWebhook::Transaction variant");
        }
    }

    #[test]
    fn test_verification_webhook_maps_to_none() {
        let data: serde_json::Value = serde_json::from_str(include_str!("../../../testdata/paybis/webhook_transaction_no_changes.json")).unwrap();

        let result = map_process_webhook(data);
        assert!(matches!(result, FiatWebhook::None), "Verification webhooks should map to FiatWebhook::None");
    }


    #[test]
    fn test_paybis_limits_parsing() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let limits: PaybisData<Vec<PaymentMethodWithLimits>> = serde_json::from_str(include_str!("../../../testdata/paybis/assets_with_limits.json"))?;

        let test_currencies = vec![
            PaybisCurrency {
                code: "USDT-TRC20".to_string(),
                blockchain_name: Some("tron".to_string()),
            },
            PaybisCurrency {
                code: "TRX".to_string(),
                blockchain_name: Some("tron".to_string()),
            },
            PaybisCurrency {
                code: "XRP".to_string(),
                blockchain_name: Some("xrp".to_string()),
            },
        ];

        let mapped_assets = map_assets_with_limits(test_currencies, &limits);

        // Test that assets with limits have expected min/max amounts
        let usdt_trc20 = mapped_assets.iter().find(|a| a.symbol == "USDT-TRC20").expect("USDT-TRC20 should exist");
        assert!(!usdt_trc20.buy_limits.is_empty(), "USDT-TRC20 should have buy limits");

        // Find USD limit
        let usd_limit = usdt_trc20.buy_limits.iter().find(|limit| limit.currency == Currency::USD);
        assert!(usd_limit.is_some(), "Should have USD limit");

        if let Some(limit) = usd_limit {
            assert_eq!(limit.min_amount, Some(5.0));
            assert_eq!(limit.max_amount, Some(20000.0));
            assert_eq!(limit.payment_type, PaymentType::Card);
        }

        Ok(())
    }
}
