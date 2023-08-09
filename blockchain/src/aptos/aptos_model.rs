use typeshare::typeshare;
//use serde::{Serialize, Deserialize};

#[typeshare]
struct AptosResource<T> {
    r#type: String,
    data: T
}

#[typeshare]
struct AptosResourceBalance {
    coin: AptosResourceCoin
}

#[typeshare]
struct AptosResourceCoin {
    value: String
}
