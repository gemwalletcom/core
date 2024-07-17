#[typeshare]
struct TonMasterchainInfo {
    last: TonBlock,
    //init: TonBlock,
}

#[typeshare]
struct TonBlock {
    seqno: i32,
    root_hash: String,
}
