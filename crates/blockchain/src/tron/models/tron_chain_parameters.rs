//TODO: Need to support u64 by typeshare
type Int64 = i64;
#[typeshare(swift = "Sendable")]
struct TronChainParameters {
    chainParameter: Vec<TronChainParameter>,
}

#[typeshare(swift = "Sendable")]
struct TronChainParameter {
    key: String,
    value: Option<Int64>,
}

#[typeshare(swift = "Equatable, Sendable")]
pub enum TronChainParameterKey {
    getCreateNewAccountFeeInSystemContract,
    getCreateAccountFee,
    getEnergyFee,
}
