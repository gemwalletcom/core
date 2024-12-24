#[typeshare(swift = "Sendable")]
struct CardanoUTXO {
    index: i32,
    address: String,
    transaction: CardanoTransactionId,
    value: CardanoAdaValue,
}

#[typeshare(swift = "Sendable")]
struct CardanoTransactionId {
    id: String,
}

#[typeshare(swift = "Sendable")]
struct CardanoAdaValue {
    ada: CardanoAdaLovelace,
}

#[typeshare(swift = "Sendable")]
struct CardanoAdaLovelace {
    lovelace: UInt64,
}

#[typeshare(swift = "Sendable")]
struct CardanoBlockTip {
    slot: i32,
}

#[typeshare(swift = "Sendable")]
struct CardanoTransactionBroadcast {
    transaction: CardanoTransactionId,
}

#[typeshare(swift = "Sendable")]
struct Cardano {
    updatableParameters: CardanoTransactionId,
}

#[typeshare(swift = "Sendable")]
#[serde(rename_all = "camelCase")]
struct CardanoGenesisConfiguration {
    updatable_parameters: CardanoUpdatableParameters,
}

#[typeshare(swift = "Sendable")]
#[serde(rename_all = "camelCase")]
struct CardanoUpdatableParameters {
    min_fee_constant: CardanoAdaValue,
    min_fee_coefficient: i32,
    network_magic: i32,
}
