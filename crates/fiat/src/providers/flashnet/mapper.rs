use std::collections::HashMap;

use number_formatter::BigNumberFormatter;
use primitives::currency::Currency;
use primitives::fiat_assets::FiatAssetLimits;
use primitives::{AssetId, Chain, FiatQuoteType, FiatTransaction, FiatTransactionStatus, PaymentType};
use streamer::FiatWebhook;

use crate::model::{FiatProviderAsset, filter_token_id};

use super::{
    client::FlashnetClient,
    model::{FlashnetOnrampResponse, FlashnetOrder, FlashnetRoute, FlashnetWebhookPayload},
};

fn map_chain(chain: &str) -> Option<Chain> {
    match chain {
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
    routes.into_iter().filter_map(map_asset).collect()
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

pub fn map_crypto_amount(estimated_out: &str, decimals: u32) -> f64 {
    BigNumberFormatter::value_as_f64(estimated_out, decimals).unwrap_or(0.0)
}

pub fn map_redirect_url(response: FlashnetOnrampResponse) -> String {
    response.payment_links.cash_app
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

pub fn map_order(order: FlashnetOrder) -> FiatTransaction {
    let chain = order.destination_chain().and_then(map_chain);
    let asset_id = chain.map(AssetId::from_chain);
    let symbol = order.destination_asset().map(str::to_ascii_uppercase).unwrap_or_default();
    let fiat_amount = order
        .amount_out
        .as_deref()
        .and_then(|value| BigNumberFormatter::value_as_f64(value, USDB_DECIMALS).ok())
        .unwrap_or_default();

    FiatTransaction {
        asset_id,
        transaction_type: FiatQuoteType::Buy,
        provider_id: FlashnetClient::NAME,
        status: map_status(&order.status),
        country: None,
        symbol,
        fiat_amount,
        fiat_currency: Currency::USD.as_ref().to_string(),
        transaction_hash: order.destination_tx_hash().map(str::to_string),
        address: order.recipient_address().map(str::to_string),
        provider_transaction_id: order.id,
    }
}

fn map_status(status: &str) -> FiatTransactionStatus {
    match status {
        "processing" | "confirming" | "bridging" | "swapping" | "awaiting_approval" | "refunding" | "delivering" => FiatTransactionStatus::Pending,
        "completed" => FiatTransactionStatus::Complete,
        "failed" | "refunded" => FiatTransactionStatus::Failed,
        other => FiatTransactionStatus::Unknown(other.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::flashnet::model::FlashnetOrder;
    use primitives::currency::Currency;
    use primitives::{AssetId, FiatQuoteType, FiatTransaction, FiatTransactionStatus};

    #[test]
    fn map_status_maps_all_documented_flashnet_statuses() {
        for status in ["processing", "confirming", "bridging", "swapping", "awaiting_approval", "refunding", "delivering"] {
            assert_eq!(map_status(status), FiatTransactionStatus::Pending);
        }

        assert_eq!(map_status("completed"), FiatTransactionStatus::Complete);
        assert_eq!(map_status("failed"), FiatTransactionStatus::Failed);
        assert_eq!(map_status("refunded"), FiatTransactionStatus::Failed);

        assert_eq!(map_status("unexpected"), FiatTransactionStatus::Unknown("unexpected".to_string()));
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
            FiatTransaction {
                asset_id: Some(AssetId::from_chain(Chain::Solana)),
                transaction_type: FiatQuoteType::Buy,
                provider_id: FlashnetClient::NAME,
                provider_transaction_id: "ord_123".to_string(),
                status: FiatTransactionStatus::Complete,
                country: None,
                symbol: "USDC".to_string(),
                fiat_amount: 1.2345,
                fiat_currency: Currency::USD.as_ref().to_string(),
                transaction_hash: Some("solana_sig_123".to_string()),
                address: Some("8gw9b9rcoW1f7a8r6Rj7H57A9x5o7oJH5Q2M5xVwqpj1".to_string()),
            }
        );
    }

    #[test]
    fn map_order_uses_nested_destination_fields() {
        let payload: serde_json::Value = serde_json::from_str(include_str!("../../../testdata/flashnet/webhook_completed.json")).unwrap();
        let order: FlashnetOrder = serde_json::from_value(payload["data"].clone()).unwrap();

        assert_eq!(
            map_order(order),
            FiatTransaction {
                asset_id: Some(AssetId::from_chain(Chain::Solana)),
                transaction_type: FiatQuoteType::Buy,
                provider_id: FlashnetClient::NAME,
                provider_transaction_id: "ord_test_completed".to_string(),
                status: FiatTransactionStatus::Complete,
                country: None,
                symbol: "USDC".to_string(),
                fiat_amount: 24.737625,
                fiat_currency: Currency::USD.as_ref().to_string(),
                transaction_hash: Some("solana_test_signature_completed".to_string()),
                address: Some("solana_test_recipient_completed".to_string()),
            }
        );
    }

    #[test]
    fn map_order_defaults_missing_amount_out_to_zero() {
        let payload: serde_json::Value = serde_json::from_str(include_str!("../../../testdata/flashnet/webhook_pending.json")).unwrap();
        let order: FlashnetOrder = serde_json::from_value(payload["data"].clone()).unwrap();

        assert_eq!(
            map_order(order),
            FiatTransaction {
                asset_id: Some(AssetId::from_chain(Chain::Solana)),
                transaction_type: FiatQuoteType::Buy,
                provider_id: FlashnetClient::NAME,
                provider_transaction_id: "ord_test_pending".to_string(),
                status: FiatTransactionStatus::Pending,
                country: None,
                symbol: "USDC".to_string(),
                fiat_amount: 0.0,
                fiat_currency: Currency::USD.as_ref().to_string(),
                transaction_hash: None,
                address: Some("solana_test_recipient_processing".to_string()),
            }
        );
    }
}
