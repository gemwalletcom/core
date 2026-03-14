use std::collections::HashMap;

use number_formatter::BigNumberFormatter;
use primitives::currency::Currency;
use primitives::{AssetId, Chain, FiatQuoteType, FiatTransaction, FiatTransactionStatus};

use crate::model::{FiatProviderAsset, filter_token_id};

use super::{
    client::FlashnetClient,
    model::{FlashnetOnrampResponse, FlashnetOrder, FlashnetRoute},
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
        buy_limits: vec![],
        sell_limits: vec![],
    })
}

const USDB_DECIMALS: u32 = 6;

pub fn map_source_amount(fiat_amount: f64) -> String {
    let amount = fiat_amount * 10f64.powi(USDB_DECIMALS as i32);
    (amount.round() as u64).to_string()
}

pub fn map_crypto_amount(estimated_out: &str, decimals: u32) -> f64 {
    BigNumberFormatter::value_as_f64(estimated_out, decimals).unwrap_or(0.0)
}

pub fn map_redirect_url(response: FlashnetOnrampResponse) -> String {
    response.payment_links.cash_app
}

pub fn map_order(order: FlashnetOrder) -> FiatTransaction {
    let chain = order.destination_chain.as_deref().and_then(map_chain);
    let asset_id = chain.map(AssetId::from_chain);
    let symbol = order.destination_asset.as_deref().map(str::to_ascii_uppercase).unwrap_or_default();
    let fiat_amount = order
        .amount_out
        .as_deref()
        .and_then(|value| BigNumberFormatter::value_as_f64(value, USDB_DECIMALS).ok())
        .unwrap_or_default();

    FiatTransaction {
        asset_id,
        transaction_type: FiatQuoteType::Buy,
        provider_id: FlashnetClient::NAME.id(),
        provider_transaction_id: order.id,
        status: map_status(&order.status),
        country: None,
        symbol,
        fiat_amount,
        fiat_currency: Currency::USD.as_ref().to_string(),
        transaction_hash: order.destination.and_then(|destination| destination.tx_hash),
        address: order.recipient_address,
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
    use crate::providers::flashnet::model::FlashnetWebhookPayload;
    use primitives::FiatTransactionStatus;
    use streamer::FiatWebhook;

    #[test]
    fn map_status_maps_pending_complete_and_failed_states() {
        match map_status("processing") {
            FiatTransactionStatus::Pending => {}
            status => panic!("Expected pending status, got {:?}", status),
        }
        match map_status("completed") {
            FiatTransactionStatus::Complete => {}
            status => panic!("Expected complete status, got {:?}", status),
        }
        match map_status("refunded") {
            FiatTransactionStatus::Failed => {}
            status => panic!("Expected failed status, got {:?}", status),
        }
        match map_status("unexpected") {
            FiatTransactionStatus::Unknown(value) => assert_eq!(value, "unexpected"),
            status => panic!("Expected unknown status, got {:?}", status),
        }
    }

    #[test]
    fn map_webhook_returns_order_id() {
        let data: serde_json::Value = serde_json::from_str(r#"{"event":"order.completed","timestamp":"2026-03-13T00:00:00Z","data":{"id":"ord_123"}}"#).unwrap();
        let payload: FlashnetWebhookPayload = serde_json::from_value(data).unwrap();

        assert!(payload.event.starts_with("order."));
        match FiatWebhook::OrderId(payload.data.id) {
            FiatWebhook::OrderId(order_id) => assert_eq!(order_id, "ord_123"),
            payload => panic!("Expected order id webhook, got {:?}", payload),
        }
    }
}
