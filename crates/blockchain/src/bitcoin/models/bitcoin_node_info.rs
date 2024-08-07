#[typeshare]
struct BitcoinNodeInfo {
    blockbook: BitcoinBlockbook,
}

#[typeshare]
struct BitcoinBlockbook {
    #[serde(rename = "inSync")]
    in_sync: bool,
    #[serde(rename = "lastBlockTime")]
    last_block_time: String,
    #[serde(rename = "bestHeight")]
    best_height: Int,
}
