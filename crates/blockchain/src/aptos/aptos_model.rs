use typeshare::typeshare;
//use serde::{Serialize, Deserialize};

#[typeshare]
#[allow(dead_code)]
struct AptosResource<T> {
    r#type: String,
    data: T,
}

#[typeshare]
#[allow(dead_code)]
struct AptosResourceBalance {
    coin: AptosResourceCoin,
}

#[typeshare]
#[allow(dead_code)]
struct AptosResourceCoin {
    value: String,
}

#[typeshare]
#[allow(dead_code)]
struct AptosAccount {
    sequence_number: String,
}

#[typeshare]
#[allow(dead_code)]
struct AptosTransaction {
    success: bool,
}

#[typeshare]
#[allow(dead_code)]
struct AptosTransactionBroacast {
    hash: String,
}

#[typeshare]
#[allow(dead_code)]
struct AptosGasFee {
    gas_estimate: i32,
    prioritized_gas_estimate: i32,
}
