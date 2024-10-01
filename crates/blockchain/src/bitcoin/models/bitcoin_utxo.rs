#[typeshare(swift = "Sendable")]
struct BitcoinUTXO {
    txid: String,
    vout: i32,
    value: String,
}
