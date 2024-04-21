#[typeshare]
struct TonTransaction {
    transaction_id: TonTransactionId,
}

#[typeshare]
struct TonTransactionId {
    hash: String,
}

#[typeshare]
struct TonTransactionMessage {
    hash: String,
}

#[typeshare]
struct TonJettonToken {
    jetton_content: TonJettonTokenContent,
}

#[typeshare]
struct TonJettonTokenContent {
    #[serde(rename = "type")]
    content_type: String,
    data: TonJettonTokenContentData,
}

#[typeshare]
struct TonJettonTokenContentData {
    name: String,
    symbol: String,
    decimals: String,
}
