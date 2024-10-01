#[typeshare(swift = "Sendable")]
struct BitcoinNodeInfo {
    blockbook: BitcoinBlockbook,
}

#[typeshare(swift = "Sendable")]
struct BitcoinBlockbook {
    #[serde(rename = "inSync")]
    in_sync: bool,
    #[serde(rename = "lastBlockTime")]
    last_block_time: String,
    #[serde(rename = "bestHeight")]
    best_height: Int,
}
