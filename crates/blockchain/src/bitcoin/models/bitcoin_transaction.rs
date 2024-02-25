#[typeshare]
struct BitcoinTransaction {
    #[serde(rename = "blockHeight")]
    block_height: i32,
}

#[typeshare]
struct BitcoinTransactionBroacastResult {
    error: Option<BitcoinTransactionBroacastError>,
    result: Option<String>,
}

#[typeshare]
struct BitcoinTransactionBroacastError {
    message: String,
}
