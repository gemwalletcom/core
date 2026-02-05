use primitives::currency::Currency;
use primitives::{AssetId, Chain, FiatProviderName, FiatQuoteType, FiatTransaction, FiatTransactionStatus, PaymentType};
use streamer::FiatWebhook;

use crate::model::FiatProviderAsset;
use primitives::fiat_assets::FiatAssetLimits;

use super::{
    client::PaybisClient,
    models::{Currency as PaybisCurrency, PaybisData, PaybisWebhookData},
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

        "USDC-SOL" => Some(AssetId::token(Chain::Solana, "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v")),
        "USDT-SOL" => Some(AssetId::token(Chain::Solana, "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB")),
        "BONK-SOL" => Some(AssetId::token(Chain::Solana, "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263")),

        "USDT-TRC20" => Some(AssetId::token(Chain::Tron, "TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t")),

        "BNB" | "BNBSC" => Some(AssetId::from_chain(Chain::SmartChain)),
        "CAKE" => Some(AssetId::token(Chain::SmartChain, "0x0E09FaBB73Bd3Ade0a17ECC321fD13a19e81cE82")),
        "ONT" => Some(AssetId::token(Chain::SmartChain, "0xFd7B3A77848f1C2D67E05E54d78d174a0C850335")),
        "TWT" => Some(AssetId::token(Chain::SmartChain, "0x4B0F1812e5Df2A09796481Ff14017e6005508003")),
        "XEC" => Some(AssetId::token(Chain::SmartChain, "0x0Ef2e7602adD1733Bfdb17aC3094d0421B502cA3")),
        "ZIL" => Some(AssetId::token(Chain::SmartChain, "0xb86AbCb37C3A4B64f74f59301AFF131a1BEcC787")),

        "USDC" => Some(AssetId::token(Chain::Ethereum, "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48")),
        "USDT" => Some(AssetId::token(Chain::Ethereum, "0xdAC17F958D2ee523a2206206994597C13D831ec7")),
        "DAI" => Some(AssetId::token(Chain::Ethereum, "0x6B175474E89094C44Da98b954EedeAC495271d0F")),

        "LINK" => Some(AssetId::token(Chain::Ethereum, "0x514910771AF9Ca656af840dff83E8264EcF986CA")),
        "AAVE" => Some(AssetId::token(Chain::Ethereum, "0x7Fc66500c84A76Ad7e9c93437bFc5Ac33E2DDaE9")),
        "UNI" => Some(AssetId::token(Chain::Ethereum, "0x1f9840a85d5aF5bf1D1762F925BDADdC4201F984")),
        "MKR" => Some(AssetId::token(Chain::Ethereum, "0x9f8F72aA9304c8B593d555F12eF6589cC3A579A2")),
        "COMP" => Some(AssetId::token(Chain::Ethereum, "0xc00e94Cb662C3520282E6f5717214004A7f26888")),
        "CRV" => Some(AssetId::token(Chain::Ethereum, "0xD533a949740bb3306d119CC777fa900bA034cd52")),
        "LDO" => Some(AssetId::token(Chain::Ethereum, "0x5A98FcBEA516Cf06857215779Fd812CA3beF1B32")),
        "ENS" => Some(AssetId::token(Chain::Ethereum, "0xC18360217D8F7Ab5e7c516566761Ea12Ce7F9D72")),
        "SUSHI" => Some(AssetId::token(Chain::Ethereum, "0x6B3595068778DD592e39A122f4f5a5cF09C90fE2")),

        "SHIB" => Some(AssetId::token(Chain::Ethereum, "0x95aD61b0a150d79219dCF64E1E6Cc01f0B64C4cE")),
        "PEPE" => Some(AssetId::token(Chain::Ethereum, "0x6982508145454Ce325dDbE47a25d4ec3d2311933")),
        "APE" => Some(AssetId::token(Chain::Ethereum, "0x4d224452801ACEd8B2F0aebE155379bb5D594381")),
        "SAND" => Some(AssetId::token(Chain::Ethereum, "0x3845badAde8e6dFF049820680d1F14bD3903a5d0")),
        "BAT" => Some(AssetId::token(Chain::Ethereum, "0x0D8775F648430679A709E98d2b0Cb6250d2887EF")),
        "FET" => Some(AssetId::token(Chain::Ethereum, "0xaea46A60368A7bD060eec7DF8CBa43b7EF41Ad85")),
        "IMX" => Some(AssetId::token(Chain::Ethereum, "0xF57e7e7C23978C3cAEC3C3548E3D615c346e79fF")),
        "CHZ" => Some(AssetId::token(Chain::Ethereum, "0x3506424F91fD33084466F402d5D97f05F8e3b4AF")),
        "AXS" => Some(AssetId::token(Chain::Ethereum, "0xBB0E17EF65F82Ab018d8EDd776e8DD940327B28b")),
        "DYDX" => Some(AssetId::token(Chain::Ethereum, "0x92D6C1e31e14520e676a687F0a93788B716BEff5")),
        "ONEINCH" => Some(AssetId::token(Chain::Ethereum, "0x111111111117dC0aa78b770fA6A738034120C302")),
        "GNO" => Some(AssetId::token(Chain::Ethereum, "0x6810e776880C02933D47DB1b9fc05908e5386b96")),
        "QNT" => Some(AssetId::token(Chain::Ethereum, "0x4a220E6096B25EADb88358cb44068A3248254675")),
        "NEXO" => Some(AssetId::token(Chain::Ethereum, "0xB62132e35a6c13ee1EE0f84dC5d40bad8d815206")),
        "HOT" => Some(AssetId::token(Chain::Ethereum, "0x6c6EE5e31d828De241282B9606C8e98Ea48526E2")),
        "ACH" => Some(AssetId::token(Chain::Ethereum, "0xEd04915c23f00A313a544955524EB7DBD823143d")),
        "AMP" => Some(AssetId::token(Chain::Ethereum, "0xfF20817765cB7f73d4bde2e66e067E58D11095C2")),
        "ANKR" => Some(AssetId::token(Chain::Ethereum, "0x8290333ceF9e6D528dD5618Fb97a76f268f3EDD4")),
        "AUDIO" => Some(AssetId::token(Chain::Ethereum, "0x18aAA7115705e8be94bfFEBDE57Af9BFc265B998")),
        "BICO" => Some(AssetId::token(Chain::Ethereum, "0xF17e65822b568B3903685a7c9F496CF7656Cc6C2")),
        "CELR" => Some(AssetId::token(Chain::Ethereum, "0x4F9254C83EB525f9FCf346490bbb3ed28a81C667")),
        "CVX" => Some(AssetId::token(Chain::Ethereum, "0x4e3FBD56CD56c3e72c1403e103b45Db9da5B9D2B")),
        "FLUX" => Some(AssetId::token(Chain::Ethereum, "0x469eDA64aEd3A3Ad6f868c44564291aA415cB1d9")),
        "FXS" => Some(AssetId::token(Chain::Ethereum, "0x3432B6A60D23Ca0dFCa7761B7ab56459D9C964D0")),
        "GLM" => Some(AssetId::token(Chain::Ethereum, "0x7DD9c5Cba05E151C895FDe1CF355C9A1D5DA6429")),
        "GTC" => Some(AssetId::token(Chain::Ethereum, "0xDe30da39c46104798bB5aA3fe8B9e0e1F348163F")),
        "ILV" => Some(AssetId::token(Chain::Ethereum, "0x767FE9EDC9E0dF98E07454847909b5E959D7ca0E")),
        "JASMY" => Some(AssetId::token(Chain::Ethereum, "0x7420B4b9a0110cdC71fB720908340C03F9Bc03EC")),
        "KNC" => Some(AssetId::token(Chain::Ethereum, "0xdd974D5C2e2928deA5F71b9825b8b646686BD200")),
        "LPT" => Some(AssetId::token(Chain::Ethereum, "0x58b6A8A3302369DAEc383334672404Ee733aB239")),
        "MASK" => Some(AssetId::token(Chain::Ethereum, "0x69af81e73A73B40adF4f3d4223Cd9b1ECE623074")),
        "NMR" => Some(AssetId::token(Chain::Ethereum, "0x1776e1F26f98b1A5dF9cD347953a26dd3Cb46671")),
        "PERP" => Some(AssetId::token(Chain::Ethereum, "0xbC396689893D065F41bc2C6EcbeE5e0085233447")),
        "PUNDIX" => Some(AssetId::token(Chain::Ethereum, "0x0FD10b9899882a6f2fcb5c371E17e70FdEe00C38")),
        "RPL" => Some(AssetId::token(Chain::Ethereum, "0xD33526068D116cE69F19A9ee46F0bd304F21A51f")),
        "SKL" => Some(AssetId::token(Chain::Ethereum, "0x00c83aeCC790e8a4453e5dD3B0B4b3680501a7A7")),
        "SSV" => Some(AssetId::token(Chain::Ethereum, "0x9D65fF81a3c488d585bBfb0Bfe3c7707c7917f54")),
        "STG" => Some(AssetId::token(Chain::Ethereum, "0xAf5191B0De278C7286d6C7CC6ab6BB8A73bA2Cd6")),
        "STORJ" => Some(AssetId::token(Chain::Ethereum, "0xB64ef51C888972c908CFacf59B47C1AfBC0Ab8aC")),
        "SYN" => Some(AssetId::token(Chain::Ethereum, "0x0f2D719407FdBeFF09D87557AbB7232601FD9F29")),
        "T" => Some(AssetId::token(Chain::Ethereum, "0xCdF7028ceAB81fA0C6971208e83fa7872994bEE5")),
        "WOO" => Some(AssetId::token(Chain::Ethereum, "0x4691937a7508860F876c9c0a2a617E7d9E945D4B")),
        "YFI" => Some(AssetId::token(Chain::Ethereum, "0x0bc529c00C6401aEF6D220BE8C6Ea1667F6Ad93e")),

        "ARB" => Some(AssetId::token(Chain::Arbitrum, "0x912CE59144191C1204E64559FE8253a0e49E6548")),
        "OP" => Some(AssetId::token(Chain::Optimism, "0x4200000000000000000000000000000000000042")),

        "USDC-STELLAR" => Some(AssetId::token(Chain::Stellar, "GA5ZSEJYB37JRC5AVCIA5MOP4RHTM335X2KGX3IHOJAPP5RE34K4KZVN::USDC")),

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
        country: webhook_data.payment.as_ref().and_then(|p| p.card.as_ref()).map(|c| c.billing_address.country.code.clone()),
        fiat_amount: webhook_data.amount_from.amount.parse().unwrap_or(0.0),
        fiat_currency: webhook_data.amount_from.currency.to_uppercase(),
        transaction_hash: webhook_data.payout.as_ref().and_then(|p| p.transaction_hash.clone()),
        address: webhook_data.payout.as_ref().and_then(|p| p.destination_wallet_address.clone()),
    })
}

fn default_limits() -> Vec<FiatAssetLimits> {
    vec![FiatAssetLimits {
        currency: Currency::USD,
        payment_type: PaymentType::Card,
        min_amount: None,
        max_amount: None,
    }]
}

use std::collections::HashSet;

pub fn map_assets(buy_currencies: Vec<PaybisCurrency>, sell_codes: HashSet<String>) -> Vec<FiatProviderAsset> {
    buy_currencies
        .into_iter()
        .filter_map(|currency| {
            if !currency.is_crypto() {
                return None;
            }
            let asset = map_asset_id(currency.clone());
            let is_sell = sell_codes.contains(&currency.code);
            let buy_limits = default_limits();
            let sell_limits = if is_sell { default_limits() } else { vec![] };

            Some(FiatProviderAsset {
                id: currency.code.clone(),
                provider: FiatProviderName::Paybis,
                chain: asset.as_ref().map(|x| x.chain),
                token_id: asset.as_ref().and_then(|x| x.token_id.clone()),
                symbol: currency.code.clone(),
                network: currency.blockchain_name.clone(),
                enabled: true,
                is_buy_enabled: true,
                is_sell_enabled: is_sell,
                unsupported_countries: Some(currency.unsupported_countries()),
                buy_limits,
                sell_limits,
            })
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
        let webhook_json: serde_json::Value = serde_json::from_str(include_str!("../../../testdata/paybis/webhook_transaction_started_no_payment.json")).unwrap();

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
    fn test_default_limits() {
        let limits = default_limits();

        assert_eq!(limits.len(), 1);
        assert_eq!(limits[0].currency, Currency::USD);
        assert_eq!(limits[0].payment_type, PaymentType::Card);
        assert_eq!(limits[0].min_amount, None);
        assert_eq!(limits[0].max_amount, None);
    }

    #[test]
    fn test_map_assets_buy_and_sell() {
        let buy_currencies = vec![
            PaybisCurrency {
                code: "ETH".to_string(),
                blockchain_name: Some("ethereum".to_string()),
            },
            PaybisCurrency {
                code: "BTC".to_string(),
                blockchain_name: Some("bitcoin".to_string()),
            },
            PaybisCurrency {
                code: "SOL".to_string(),
                blockchain_name: Some("solana".to_string()),
            },
        ];
        let sell_codes: HashSet<String> = ["ETH".to_string(), "SOL".to_string()].into_iter().collect();

        let assets = map_assets(buy_currencies, sell_codes);

        let eth = assets.iter().find(|a| a.symbol == "ETH").unwrap();
        assert!(eth.is_buy_enabled);
        assert!(eth.is_sell_enabled);

        let btc = assets.iter().find(|a| a.symbol == "BTC").unwrap();
        assert!(btc.is_buy_enabled);
        assert!(!btc.is_sell_enabled);

        let sol = assets.iter().find(|a| a.symbol == "SOL").unwrap();
        assert!(sol.is_buy_enabled);
        assert!(sol.is_sell_enabled);
    }
}
