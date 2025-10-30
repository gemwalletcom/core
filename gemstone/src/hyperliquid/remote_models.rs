use alloy_primitives::hex;
use gem_hypercore::core::actions;

// Order types
pub type HyperPlaceOrder = actions::PlaceOrder;
pub type HyperOrder = actions::Order;
pub type HyperOrderType = actions::OrderType;
pub type HyperLimitOrder = actions::LimitOrder;
pub type HyperTrigger = actions::Trigger;
pub type HyperTimeInForce = actions::TimeInForce;
pub type HyperTpslType = actions::TpslType;
pub type HyperGrouping = actions::Grouping;
pub type HyperBuilder = actions::Builder;

// Action types
pub type HyperApproveAgent = actions::ApproveAgent;
pub type HyperApproveBuilderFee = actions::ApproveBuilderFee;
pub type HyperCDeposit = actions::CDeposit;
pub type HyperCWithdraw = actions::CWithdraw;
pub type HyperCancel = actions::Cancel;
pub type HyperCancelOrder = actions::CancelOrder;
pub type HyperSetReferrer = actions::SetReferrer;
pub type HyperSpotSend = actions::SpotSend;
pub type HyperTokenDelegate = actions::TokenDelegate;
pub type HyperUpdateLeverage = actions::UpdateLeverage;
pub type HyperUsdClassTransfer = actions::UsdClassTransfer;
pub type HyperUsdSend = actions::UsdSend;
pub type HyperWithdrawalRequest = actions::WithdrawalRequest;

pub fn hyper_make_approve_agent(name: String, address: String, nonce: u64) -> HyperApproveAgent {
    actions::ApproveAgent::new(address, name, nonce)
}

pub fn hyper_make_approve_builder(max_fee_rate: String, builder: String, nonce: u64) -> HyperApproveBuilderFee {
    actions::ApproveBuilderFee::new(max_fee_rate, builder, nonce)
}

pub fn hyper_make_market_order(asset: u32, is_buy: bool, price: String, size: String, reduce_only: bool, builder: Option<HyperBuilder>) -> HyperPlaceOrder {
    actions::make_market_order(asset, is_buy, &price, &size, reduce_only, builder)
}

pub fn hyper_make_market_with_tp_sl(
    asset: u32,
    is_buy: bool,
    price: String,
    size: String,
    reduce_only: bool,
    tp_trigger: Option<String>,
    sl_trigger: Option<String>,
    builder: Option<HyperBuilder>,
) -> HyperPlaceOrder {
    actions::make_market_with_tp_sl(asset, is_buy, &price, &size, reduce_only, tp_trigger, sl_trigger, builder)
}

pub fn hyper_build_signed_request(signature: String, action: String, timestamp: u64) -> String {
    let sig_bytes = hex::decode(&signature).unwrap();

    let r = hex::encode_prefixed(&sig_bytes[0..32]);
    let s = hex::encode_prefixed(&sig_bytes[32..64]);
    let v = sig_bytes[64] as u64;

    let action_json: serde_json::Value = serde_json::from_str(&action).unwrap();

    let signed_request = serde_json::json!({
        "action": action_json,
        "signature": {
            "r": r,
            "s": s,
            "v": v
        },
        "nonce": timestamp,
        "isFrontend": true
    });

    serde_json::to_string(&signed_request).unwrap()
}
