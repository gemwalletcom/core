#[typeshare(swift = "Sendable")]
struct TonTransaction {
    transaction_id: TonTransactionId,
}

#[typeshare(swift = "Sendable")]
struct TonTransactionId {
    hash: String,
}

#[typeshare(swift = "Sendable")]
struct TonTransactionMessage {
    hash: String,
}

#[typeshare(swift = "Sendable")]
struct TonJettonToken {
    jetton_content: TonJettonTokenContent,
}

#[typeshare(swift = "Sendable")]
struct TonJettonBalance {
    balance: UInt64,
}

#[typeshare(swift = "Sendable")]
struct TonJettonTokenContent {
    #[serde(rename = "type")]
    content_type: String,
    data: TonJettonTokenContentData,
}

#[typeshare(swift = "Sendable")]
struct TonJettonTokenContentData {
    name: String,
    symbol: String,
    decimals: String,
}
