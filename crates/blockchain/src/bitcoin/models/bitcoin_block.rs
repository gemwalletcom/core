#[typeshare(swift = "Sendable")]
struct BitcoinBlock {
    #[serde(rename = "previousBlockHash")]
    previous_block_hash: Option<String>,
}
