#[typeshare(swift = "Sendable")]
struct NearAccount {
    amount: String,
}

#[typeshare(swift = "Sendable")]
struct NearAccountAccessKey {
    nonce: Int,
}

#[typeshare(swift = "Sendable")]
struct NearRPCError {
    error: NearError,
}

#[typeshare(swift = "Sendable")]
struct NearError {
    message: String,
    data: Option<String>,
}

#[typeshare(swift = "Sendable")]
struct NearBlock {
    header: NearBlockHeader,
}

#[typeshare(swift = "Sendable")]
struct NearBlockHeader {
    hash: String,
    height: Int,
}

#[typeshare(swift = "Sendable")]
struct NearGasPrice {
    gas_price: String,
}

#[typeshare(swift = "Sendable")]
struct NearBroadcastResult {
    final_execution_status: String,
    transaction: NearBroadcastTransaction,
}

#[typeshare(swift = "Sendable")]
struct NearBroadcastTransaction {
    hash: String,
}

#[typeshare(swift = "Sendable")]
struct NearGenesisConfig {
    chain_id: String,
}
