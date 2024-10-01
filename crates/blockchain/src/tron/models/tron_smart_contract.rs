#[typeshare(swift = "Sendable")]
struct TronSmartContractCall {
    contract_address: String,
    function_selector: String,
    parameter: Option<String>,
    fee_limit: Option<u32>,
    call_value: Option<u32>,
    owner_address: String,
    visible: Option<bool>,
}

#[typeshare(swift = "Sendable")]
struct TronSmartContractResult {
    result: TronSmartContractResultMessage,
    constant_result: Vec<String>,
    energy_used: i32,
}

#[typeshare(swift = "Sendable")]
struct TronSmartContractResultMessage {
    result: bool,
    message: Option<String>,
}
