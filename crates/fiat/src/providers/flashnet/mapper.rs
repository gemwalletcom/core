use std::collections::{BTreeMap, HashMap};

use number_formatter::{BigNumberFormatter, NumberFormatterError};
use primitives::currency::Currency;
use primitives::fiat_assets::FiatAssetLimits;
use primitives::{Chain, FiatTransactionStatus, FiatTransactionUpdate, PaymentType};
use streamer::FiatWebhook;

use crate::model::{FiatProviderAsset, filter_token_id};

use super::{
    client::FlashnetClient,
    model::{FlashnetOnrampResponse, FlashnetOrder, FlashnetRoute, FlashnetWebhookPayload},
};

fn map_chain(chain: &str) -> Option<Chain> {
    match chain {
        "bitcoin" => Some(Chain::Bitcoin),
        "solana" => Some(Chain::Solana),
        "base" => Some(Chain::Base),
        "ethereum" => Some(Chain::Ethereum),
        "arbitrum" => Some(Chain::Arbitrum),
        "optimism" => Some(Chain::Optimism),
        "polygon" => Some(Chain::Polygon),
        "tron" => Some(Chain::Tron),
        "plasma" => Some(Chain::Plasma),
        _ => None,
    }
}

pub fn map_assets(routes: Vec<FlashnetRoute>) -> Vec<FiatProviderAsset> {
    routes
        .into_iter()
        .filter_map(map_asset)
        .fold(BTreeMap::new(), |mut assets, asset| {
            assets.entry(asset.id.clone()).or_insert(asset);
            assets
        })
        .into_values()
        .collect()
}

fn map_asset(route: FlashnetRoute) -> Option<FiatProviderAsset> {
    let destination = route.destination;
    let chain = map_chain(&destination.chain)?;
    let token_id = filter_token_id(Some(chain), destination.contract_address);
    let symbol = destination.asset;
    let network = destination.chain;

    Some(FiatProviderAsset {
        id: format!("{}_{}", symbol.to_ascii_lowercase(), network),
        provider: FlashnetClient::NAME,
        chain: Some(chain),
        symbol,
        token_id,
        network: Some(network),
        enabled: true,
        is_buy_enabled: true,
        is_sell_enabled: false,
        unsupported_countries: Some(HashMap::new()),
        buy_limits: vec![FiatAssetLimits {
            currency: Currency::USD,
            payment_type: PaymentType::CashApp,
            min_amount: Some(1.0),
            max_amount: Some(500.0),
        }],
        sell_limits: vec![],
    })
}

const USDB_DECIMALS: u32 = 6;

pub fn map_amount(amount: f64, decimals: u32) -> String {
    let value = amount * 10f64.powi(decimals as i32);
    (value.round() as u64).to_string()
}

pub fn map_source_amount(fiat_amount: f64) -> String {
    map_amount(fiat_amount, USDB_DECIMALS)
}

pub fn map_crypto_amount(estimated_out: &str, decimals: u32) -> Result<f64, NumberFormatterError> {
    BigNumberFormatter::value_as_f64(estimated_out, decimals)
}

pub fn map_redirect_url(response: &FlashnetOnrampResponse) -> String {
    response.payment_links.cash_app.clone()
}

pub fn map_webhook(payload: FlashnetWebhookPayload) -> FiatWebhook {
    match payload.event.as_str() {
        "order.processing"
        | "order.confirming"
        | "order.bridging"
        | "order.swapping"
        | "order.awaiting_approval"
        | "order.refunding"
        | "order.delivering"
        | "order.completed"
        | "order.failed"
        | "order.refunded" => FiatWebhook::OrderId(payload.data.id),
        _ => FiatWebhook::None,
    }
}

pub fn map_order(order: FlashnetOrder) -> FiatTransactionUpdate {
    let transaction_id = order.id.clone();
    let fiat_amount = order
        .effective_amount_out()
        .and_then(|value| BigNumberFormatter::value_as_f64(value, USDB_DECIMALS).ok())
        .unwrap_or_default();

    FiatTransactionUpdate {
        transaction_id,
        provider_transaction_id: None,
        status: map_status(&order.status),
        transaction_hash: order.destination_tx_hash().map(str::to_string),
        fiat_amount: Some(fiat_amount),
        fiat_currency: Some(Currency::USD.to_string()),
    }
}

fn map_status(status: &str) -> FiatTransactionStatus {
    match status {
        "processing" | "confirming" | "bridging" | "swapping" | "awaiting_approval" | "refunding" | "delivering" => FiatTransactionStatus::Pending,
        "completed" => FiatTransactionStatus::Complete,
        "failed" | "refunded" => FiatTransactionStatus::Failed,
        _ => FiatTransactionStatus::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::flashnet::model::{FlashnetOrder, FlashnetRoute, FlashnetRouteAsset};
    use primitives::FiatTransactionStatus;

    #[test]
    fn map_status_maps_all_documented_flashnet_statuses() {
        for status in ["processing", "confirming", "bridging", "swapping", "awaiting_approval", "refunding", "delivering"] {
            assert_eq!(map_status(status), FiatTransactionStatus::Pending);
        }

        assert_eq!(map_status("completed"), FiatTransactionStatus::Complete);
        assert_eq!(map_status("failed"), FiatTransactionStatus::Failed);
        assert_eq!(map_status("refunded"), FiatTransactionStatus::Failed);

        assert_eq!(map_status("unexpected"), FiatTransactionStatus::Unknown);
    }

    #[test]
    fn map_amount_converts_to_smallest_units() {
        assert_eq!(map_amount(100.0, 6), "100000000");
        assert_eq!(map_amount(1.0, 6), "1000000");
        assert_eq!(map_amount(0.5, 6), "500000");
        assert_eq!(map_amount(50.0, 8), "5000000000");
        assert_eq!(map_amount(500.0, 6), "500000000");
    }

    #[test]
    fn map_source_amount_uses_usdb_decimals() {
        assert_eq!(map_source_amount(100.0), "100000000");
        assert_eq!(map_source_amount(1.0), "1000000");
    }

    #[test]
    fn map_crypto_amount_parses_valid_values_and_rejects_invalid_ones() {
        assert_eq!(map_crypto_amount("1000000", 6).unwrap(), 1.0);
        assert_eq!(map_crypto_amount("500000", 6).unwrap(), 0.5);
        assert!(map_crypto_amount("invalid", 6).is_err());
    }

    #[test]
    fn map_assets_deduplicates_duplicate_destination_routes() {
        let routes = vec![
            FlashnetRoute {
                source_chain: "lightning".to_string(),
                source_asset: "BTC".to_string(),
                destination: FlashnetRouteAsset {
                    chain: "solana".to_string(),
                    asset: "USDC".to_string(),
                    contract_address: Some("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string()),
                },
            },
            FlashnetRoute {
                source_chain: "lightning".to_string(),
                source_asset: "BTC".to_string(),
                destination: FlashnetRouteAsset {
                    chain: "solana".to_string(),
                    asset: "USDC".to_string(),
                    contract_address: Some("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string()),
                },
            },
            FlashnetRoute {
                source_chain: "lightning".to_string(),
                source_asset: "BTC".to_string(),
                destination: FlashnetRouteAsset {
                    chain: "base".to_string(),
                    asset: "USDC".to_string(),
                    contract_address: Some("0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913".to_string()),
                },
            },
        ];

        let assets = map_assets(routes);

        assert_eq!(assets.len(), 2);
        assert_eq!(assets[0].id, "usdc_base");
        assert_eq!(assets[1].id, "usdc_solana");
    }

    #[test]
    fn map_assets_ignores_unsupported_usdc_chains() {
        let routes = vec![
            FlashnetRoute {
                source_chain: "lightning".to_string(),
                source_asset: "BTC".to_string(),
                destination: FlashnetRouteAsset {
                    chain: "base".to_string(),
                    asset: "USDC".to_string(),
                    contract_address: Some("0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913".to_string()),
                },
            },
            FlashnetRoute {
                source_chain: "lightning".to_string(),
                source_asset: "BTC".to_string(),
                destination: FlashnetRouteAsset {
                    chain: "tempo".to_string(),
                    asset: "USDC".to_string(),
                    contract_address: Some("0x20c000000000000000000000b9537d11c60e8b50".to_string()),
                },
            },
        ];

        let assets = map_assets(routes);

        assert_eq!(assets.len(), 1);
        assert_eq!(assets[0].id, "usdc_base");
    }

    #[test]
    fn map_webhook_ignores_non_order_events() {
        let data: serde_json::Value = serde_json::from_str(r#"{"event":"quote.updated","timestamp":"2026-03-13T00:00:00Z","data":{"id":"ord_123"}}"#).unwrap();
        let payload: FlashnetWebhookPayload = serde_json::from_value(data).unwrap();

        match map_webhook(payload) {
            FiatWebhook::None => {}
            payload => panic!("Expected ignored webhook, got {:?}", payload),
        }
    }

    #[test]
    fn map_order_supports_legacy_top_level_destination_fields() {
        let order: FlashnetOrder = serde_json::from_str(include_str!("../../../testdata/flashnet/order_completed.json")).unwrap();

        assert_eq!(
            map_order(order),
            FiatTransactionUpdate {
                transaction_id: "ord_123".to_string(),
                provider_transaction_id: None,
                status: FiatTransactionStatus::Complete,
                transaction_hash: Some("solana_sig_123".to_string()),
                fiat_amount: Some(1.2345),
                fiat_currency: Some("USD".to_string()),
            }
        );
    }

    #[test]
    fn map_order_uses_nested_destination_fields() {
        let payload: serde_json::Value = serde_json::from_str(include_str!("../../../testdata/flashnet/webhook_completed.json")).unwrap();
        let order: FlashnetOrder = serde_json::from_value(payload["data"].clone()).unwrap();

        assert_eq!(
            map_order(order),
            FiatTransactionUpdate {
                transaction_id: "ord_test_completed".to_string(),
                provider_transaction_id: None,
                status: FiatTransactionStatus::Complete,
                transaction_hash: Some("solana_test_signature_completed".to_string()),
                fiat_amount: Some(24.737625),
                fiat_currency: Some("USD".to_string()),
            }
        );
    }

    #[test]
    fn map_order_falls_back_to_payment_intent_target_amount() {
        let payload: serde_json::Value = serde_json::from_str(include_str!("../../../testdata/flashnet/webhook_pending.json")).unwrap();
        let order: FlashnetOrder = serde_json::from_value(payload["data"].clone()).unwrap();

        assert_eq!(
            map_order(order),
            FiatTransactionUpdate {
                transaction_id: "ord_test_pending".to_string(),
                provider_transaction_id: None,
                status: FiatTransactionStatus::Pending,
                transaction_hash: None,
                fiat_amount: Some(49.47525),
                fiat_currency: Some("USD".to_string()),
            }
        );
    }
}
