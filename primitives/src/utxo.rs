#[typeshare]
struct UTXO {
    transaction_id: String,
    vout: i32,
    value: String,
}