#[typeshare]
struct TonEstimateFee {
    address: String,
    body: String,
    ignore_chksig: bool,
}

#[typeshare]
struct TonFees {
    source_fees: TonFee,
}


#[typeshare]
struct TonFee {
    in_fwd_fee: i32,
    storage_fee: i32,
}