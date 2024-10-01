#[typeshare(swift = "Sendable")]
struct BitcoinTransaction {
    #[serde(rename = "blockHeight")]
    block_height: i32,
}

#[typeshare(swift = "Sendable")]
struct BitcoinTransactionBroacastResult {
    error: Option<BitcoinTransactionBroacastError>,
    result: Option<String>,
}

#[typeshare(swift = "Sendable")]
struct BitcoinTransactionBroacastError {
    message: String,
}
