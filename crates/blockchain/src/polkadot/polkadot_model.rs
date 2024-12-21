#[typeshare(swift = "Sendable")]
struct PolkadotAccountBalance {
    free: String,
    reserved: String,
    nonce: String,
}

#[typeshare(swift = "Sendable")]
struct PolkadotBlock {
    number: String,
    extrinsics: Vec<PolkadotExtrinsic>,
}

#[typeshare(swift = "Sendable")]
struct PolkadotExtrinsic {
    hash: String,
    success: bool,
}

#[typeshare(swift = "Sendable")]
struct PolkadotNodeVersion {
    chain: String,
}

#[typeshare(swift = "Sendable")]
#[serde(rename_all = "camelCase")]
struct PolkadotTransactionMaterial {
    at: PolkadotTransactionMaterialBlock,
    genesis_hash: String,
    chain_name: String,
    spec_name: String,
    spec_version: String,
    tx_version: String,
}

#[typeshare(swift = "Sendable")]
#[serde(rename_all = "camelCase")]
struct PolkadotTransactionMaterialBlock {
    height: String,
    hash: String,
}

#[typeshare(swift = "Sendable")]
#[serde(rename_all = "camelCase")]
struct PolkadotTransactionPayload {
    tx: String,
}

#[typeshare(swift = "Sendable")]
#[serde(rename_all = "camelCase")]
struct PolkadotEstimateFee {
    partial_fee: String,
}

#[typeshare(swift = "Sendable")]
struct PolkadotTransactionBroadcast {
    hash: String,
}

#[typeshare(swift = "Sendable")]
struct PolkadotTransactionBroadcastError {
    error: String,
    cause: String,
}
