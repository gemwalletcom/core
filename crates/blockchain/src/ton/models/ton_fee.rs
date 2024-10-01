#[typeshare(swift = "Sendable")]
struct TonEstimateFee {
    address: String,
    body: String,
    ignore_chksig: bool,
}

#[typeshare(swift = "Sendable")]
struct TonFees {
    source_fees: TonFee,
}

#[typeshare(swift = "Sendable")]
struct TonFee {
    in_fwd_fee: i32,
    storage_fee: i32,
}
