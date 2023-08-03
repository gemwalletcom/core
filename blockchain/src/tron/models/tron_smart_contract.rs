#[typeshare]
struct TronSmartContractCall {
    contract_address: String,
    function_selector: String,
    parameter: String,
    fee_limit: u32,
    call_value: u32,
    owner_address: String,
}

#[typeshare]
struct TronSmartContractResult {
    constant_result: Vec<String>,
}