#[typeshare(swift = "Sendable")]
struct TonMasterchainInfo {
    last: TonBlock,
    //init: TonBlock,
}

#[typeshare(swift = "Sendable")]
struct TonBlock {
    seqno: i32,
    root_hash: String,
}
