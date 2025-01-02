#[typeshare(swift = "Sendable")]
#[serde(rename_all = "camelCase")]
#[typeshare(swiftGenericConstraints = "T: Sendable")]
struct CardanoUTXOS<T> {
    utxos: T,
}

#[typeshare(swift = "Sendable")]
#[serde(rename_all = "camelCase")]
struct CardanoUTXO {
    address: String,
    tx_hash: String,
    index: i32,
    value: String,
}

#[typeshare(swift = "Sendable")]
#[serde(rename_all = "camelCase")]
struct CardanoBalance {
    address: String,
    tx_hash: String,
    index: i32,
}

#[typeshare(swift = "Sendable")]
#[serde(rename_all = "camelCase")]
struct CardanoAggregateBalance {
    aggregate: CardanoAggregateSum,
}

#[typeshare(swift = "Sendable")]
#[serde(rename_all = "camelCase")]
struct CardanoAggregateSum {
    sum: CardanoAggregateSumValue,
}

#[typeshare(swift = "Sendable")]
#[serde(rename_all = "camelCase")]
struct CardanoAggregateSumValue {
    value: Option<String>,
}

#[typeshare(swift = "Sendable")]
#[serde(rename_all = "camelCase")]
struct CardanoTransactionBroadcast {
    submit_transaction: CardanoSubmitTransactionHash,
}

#[typeshare(swift = "Sendable")]
#[serde(rename_all = "camelCase")]
struct CardanoSubmitTransactionHash {
    hash: String,
}

#[typeshare(swift = "Sendable")]
#[serde(rename_all = "camelCase")]
struct CardanoTransactions {
    transactions: Vec<CardanoTransaction>,
}

#[typeshare(swift = "Sendable")]
#[serde(rename_all = "camelCase")]
struct CardanoTransaction {
    fee: String,
    block: CardanoBlock,
}

#[typeshare(swift = "Sendable")]
#[serde(rename_all = "camelCase")]
struct CardanoBlock {
    number: i32,
}

#[typeshare(swift = "Sendable")]
#[serde(rename_all = "camelCase")]
struct CardanoBlockData {
    cardano: CardanoBlockTip,
}

#[typeshare(swift = "Sendable")]
#[serde(rename_all = "camelCase")]
struct CardanoBlockTip {
    tip: CardanoBlock,
}

#[typeshare(swift = "Sendable")]
#[serde(rename_all = "camelCase")]
struct CardanoGenesisData {
    genesis: CardanoGenesis,
}

#[typeshare(swift = "Sendable")]
#[serde(rename_all = "camelCase")]
struct CardanoGenesis {
    shelley: CardanoGenesisShelley,
}

#[typeshare(swift = "Sendable")]
#[serde(rename_all = "camelCase")]
struct CardanoGenesisShelley {
    network_magic: i32,
}
