#[typeshare]
struct TronSmartContractCall {
    contract_address: String,
    function_selector: String,
    parameter: String,
    fee_limit: u32,
    call_value: u32,
    owner_address: String,
    visible: bool,
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
