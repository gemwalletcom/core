#[typeshare(swift = "Sendable")]
struct TronBlock {
    #[serde(rename = "block_header")]
    block_header: TronHeaderRawData,
}

#[typeshare(swift = "Sendable")]
struct TronHeaderRawData {
    raw_data: TronHeader,
}
//TODO: Need to support u64 by typeshare
type UInt64 = u64;
#[typeshare(swift = "Sendable")]
struct TronHeader {
    number: UInt64,
    version: UInt64,
    #[serde(rename = "txTrieRoot")]
    tx_trie_root: String,
    witness_address: String,
    #[serde(rename = "parentHash")]
    parent_hash: String,
    timestamp: UInt64,
}
