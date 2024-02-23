//TODO: Need to support u64 by typeshare
type Int64 = i64;
#[typeshare]
struct TronChainParameters {
    chainParameter: Vec<TronChainParameter>,
}

#[typeshare]
struct TronChainParameter {
    key: String,
    value: Option<Int64>,
}

#[typeshare(swift = "Equatable, Codable")]
pub enum TronChainParameterKey {
    getCreateNewAccountFeeInSystemContract,
    getCreateAccountFee,
    getEnergyFee,
}