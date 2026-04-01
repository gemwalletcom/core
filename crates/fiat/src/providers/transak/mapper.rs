use super::models::{Asset, FiatCurrency, TransakOrderResponse};
use crate::model::{FiatProviderAsset, filter_token_id};
use primitives::PaymentType;
use primitives::currency::Currency;
use primitives::fiat_assets::FiatAssetLimits;
use primitives::{Chain, FiatProviderName, FiatQuoteType, FiatTransactionStatus, FiatTransactionUpdate};

pub fn map_asset_chain(network: &str, coin_id: Option<&str>) -> Option<Chain> {
    match network {
        "ethereum" => Some(Chain::Ethereum),
        "polygon" => Some(Chain::Polygon),
        "aptos" => Some(Chain::Aptos),
        "sui" => Some(Chain::Sui),
        "arbitrum" => Some(Chain::Arbitrum),
        "optimism" => Some(Chain::Optimism),
        "base" => Some(Chain::Base),
        "bsc" => Some(Chain::SmartChain),
        "tron" => Some(Chain::Tron),
        "solana" => Some(Chain::Solana),
        "avaxcchain" => Some(Chain::AvalancheC),
        "ton" => Some(Chain::Ton),
        "osmosis" => Some(Chain::Osmosis),
        "fantom" => Some(Chain::Fantom),
        "injective" => Some(Chain::Injective),
        "sei" => Some(Chain::Sei),
        "linea" => Some(Chain::Linea),
        "zksync" => Some(Chain::ZkSync),
        "celo" => Some(Chain::Celo),
        "mantle" => Some(Chain::Mantle),
        "opbnb" => Some(Chain::OpBNB),
        "unichain" => Some(Chain::Unichain),
        "stellar" => Some(Chain::Stellar),
        "algorand" => Some(Chain::Algorand),
        "berachain" => Some(Chain::Berachain),
        "hyperevm" => Some(Chain::Hyperliquid),
        "hyperliquid" => Some(Chain::HyperCore),
        "monad" => Some(Chain::Monad),
        "plasma" => Some(Chain::Plasma),
        "mainnet" => match coin_id? {
            "bitcoin" => Some(Chain::Bitcoin),
            "litecoin" => Some(Chain::Litecoin),
            "ripple" => Some(Chain::Xrp),
            "dogecoin" => Some(Chain::Doge),
            "tron" => Some(Chain::Tron),
            "cosmos" => Some(Chain::Cosmos),
            "near" => Some(Chain::Near),
            "stellar" => Some(Chain::Stellar),
            "algorand" => Some(Chain::Algorand),
            "polkadot" => Some(Chain::Polkadot),
            "cardano" => Some(Chain::Cardano),
            _ => None,
        },
        _ => None,
    }
}

fn map_status(status: &str) -> FiatTransactionStatus {
    match status {
        "ORDER_PAYMENT_VERIFYING" | "PAYMENT_DONE_MARKED_BY_USER" | "PENDING_DELIVERY_FROM_TRANSAK" | "AWAITING_PAYMENT_FROM_USER" | "PROCESSING" => FiatTransactionStatus::Pending,
        "EXPIRED" | "FAILED" | "CANCELLED" | "REFUNDED" => FiatTransactionStatus::Failed,
        "COMPLETED" => FiatTransactionStatus::Complete,
        _ => FiatTransactionStatus::Unknown,
    }
}

pub fn map_order_from_response(payload: TransakOrderResponse) -> FiatTransactionUpdate {
    let transaction_id = payload.quote_id.clone().unwrap_or_else(|| payload.id.clone());
    let provider_transaction_id = (transaction_id != payload.id).then_some(payload.id.clone());

    FiatTransactionUpdate {
        transaction_id,
        provider_transaction_id,
        status: map_status(&payload.status),
        transaction_hash: payload.transaction_hash,
        fiat_amount: Some(payload.fiat_amount),
        fiat_currency: Some(payload.fiat_currency.to_ascii_uppercase()),
    }
}
fn map_limits(fiat_currencies: &[FiatCurrency], quote_type: FiatQuoteType) -> Vec<FiatAssetLimits> {
    fiat_currencies
        .iter()
        .filter_map(|fiat_currency| fiat_currency.symbol.parse::<Currency>().ok().map(|currency| (currency, fiat_currency)))
        .flat_map(|(currency, fiat_currency)| {
            fiat_currency
                .payment_options
                .iter()
                .filter_map(|payment_option| {
                    if !payment_option.is_active {
                        return None;
                    }
                    let payment_type = map_payment_type(&payment_option.id)?;
                    let (min_amount, max_amount) = match quote_type {
                        FiatQuoteType::Buy => (payment_option.min_amount, payment_option.max_amount),
                        FiatQuoteType::Sell => (payment_option.min_amount_for_pay_out, payment_option.max_amount_for_pay_out),
                    };
                    Some(FiatAssetLimits {
                        currency: currency.clone(),
                        payment_type,
                        min_amount,
                        max_amount,
                    })
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

pub fn map_asset(asset: Asset) -> Option<FiatProviderAsset> {
    let chain = map_asset_chain(&asset.network.name, Some(&asset.coin_id));
    let token_id = filter_token_id(chain, asset.clone().address);
    let enabled = asset.is_allowed && !asset.is_suspended.unwrap_or(false);
    let is_sell_enabled = asset.is_pay_in_allowed.unwrap_or(false);

    Some(FiatProviderAsset {
        id: asset.clone().unique_id,
        provider: FiatProviderName::Transak,
        chain,
        token_id,
        symbol: asset.clone().symbol,
        network: Some(asset.clone().network.name),
        enabled,
        is_buy_enabled: true,
        is_sell_enabled,
        unsupported_countries: Some(asset.unsupported_countries()),
        buy_limits: vec![],
        sell_limits: vec![],
    })
}

pub fn map_asset_with_limits(asset: Asset, fiat_currencies: &[FiatCurrency]) -> Option<FiatProviderAsset> {
    let provider_asset = map_asset(asset)?;
    let buy_limits = map_limits(fiat_currencies, FiatQuoteType::Buy);
    let sell_limits = map_limits(fiat_currencies, FiatQuoteType::Sell);
    let is_buy_enabled = !buy_limits.is_empty();
    let is_sell_enabled = provider_asset.is_sell_enabled && !sell_limits.is_empty();
    Some(FiatProviderAsset {
        buy_limits,
        sell_limits,
        is_buy_enabled,
        is_sell_enabled,
        ..provider_asset
    })
}

fn map_payment_type(payment_id: &str) -> Option<PaymentType> {
    match payment_id {
        "credit_debit_card" => Some(PaymentType::Card),
        "apple_pay" => Some(PaymentType::ApplePay),
        "google_pay" => Some(PaymentType::GooglePay),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::transak::models::{Data, FiatCurrency, Response, TransakOrderResponse};
    use primitives::{FiatTransactionStatus, PaymentType};

    #[test]
    fn test_map_order_buy_failed() {
        let response: Data<TransakOrderResponse> = serde_json::from_str(include_str!("../../../testdata/transak/transaction_buy_error.json")).expect("Failed to parse test data");

        let result = map_order_from_response(response.data);

        assert_eq!(result.transaction_id, "e75764cd-1275-476e-b6fa-9af787b40974");
        assert_eq!(result.provider_transaction_id, Some("df7997b7-a19f-447e-b9fe-2f0eb7cb7b3a".to_string()));
        assert_eq!(result.status, FiatTransactionStatus::Failed);
        assert_eq!(result.fiat_amount, Some(108.0));
        assert_eq!(result.fiat_currency, Some("USD".to_string()));
    }

    #[test]
    fn test_map_asset_with_limits() {
        let fiat_response: Response<Vec<FiatCurrency>> = serde_json::from_str(include_str!("../../../testdata/transak/fiat_currencies.json")).unwrap();

        use crate::providers::transak::models::{Asset, AssetNetwork};
        let asset = Asset {
            coin_id: "ethereum".to_string(),
            unique_id: "eth".to_string(),
            symbol: "ETH".to_string(),
            network: AssetNetwork { name: "ethereum".to_string() },
            address: None,
            is_allowed: true,
            is_suspended: Some(false),
            is_pay_in_allowed: Some(true),
            kyc_countries_not_supported: vec![],
        };

        let result = map_asset_with_limits(asset, &fiat_response.response).unwrap();

        assert_eq!(result.symbol, "ETH");
        assert!(result.enabled);
        assert!(!result.buy_limits.is_empty());

        let card_limit = result.buy_limits.iter().find(|limit| limit.payment_type == PaymentType::Card).unwrap();
        assert_eq!(card_limit.min_amount, Some(5.0));
        assert_eq!(card_limit.max_amount, Some(3000.0));

        let googlepay_limit = result.buy_limits.iter().find(|limit| limit.payment_type == PaymentType::GooglePay).unwrap();
        assert_eq!(googlepay_limit.min_amount, Some(30.0));
        assert_eq!(googlepay_limit.max_amount, Some(1500.0));
    }
}
