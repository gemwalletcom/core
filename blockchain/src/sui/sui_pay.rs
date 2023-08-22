#[typeshare]
#[serde(rename_all = "camelCase")]
struct SuiPay {
    tx_bytes: String,
}

#[typeshare]
#[serde(rename_all = "camelCase")]
struct SuiPayRequest {
    sender_address: String,
    recipient_address: String,
    coins: Vec<String>,
    amount: String,
    gas_budget: String,
}

#[typeshare]
#[serde(rename_all = "camelCase")]
struct SuiBroadcastTransaction {
    digest: String,
}
