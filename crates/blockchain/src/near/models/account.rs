#[typeshare]
struct NearAccount {
    amount: String,
}

#[typeshare]
struct NearAccountAccessKey {
    nonce: Int,
}

#[typeshare]
struct NearRPCError {
    error: NearError,
}

#[typeshare]
struct NearError {
    message: String,
    data: Option<String>,
}

#[typeshare]
struct NearBlock {
    //height: i32,
    header: NearBlockHeader,
}

#[typeshare]
struct NearBlockHeader {
    hash: String,
}

#[typeshare]
struct NearGasPrice {
    gas_price: String,
}

#[typeshare]
struct NearBroadcastResult {
    final_execution_status: String,
    transaction: NearBroadcastTransaction,
}

#[typeshare]
struct NearBroadcastTransaction {
    hash: String,
}
