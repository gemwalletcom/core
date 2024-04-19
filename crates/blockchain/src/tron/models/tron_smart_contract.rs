#[typeshare]
struct TronSmartContractCall {
    contract_address: String,
    function_selector: String,
    parameter: Option<String>,
    fee_limit: Option<u32>,
    call_value: Option<u32>,
    owner_address: String,
    visible: Option<bool>,
}

#[typeshare]
struct TronSmartContractResult {
    result: TronSmartContractResultMessage,
    constant_result: Vec<String>,
    energy_used: i32,
}

#[typeshare]
struct TronSmartContractResultMessage {
    result: bool,
    message: Option<String>,
}
