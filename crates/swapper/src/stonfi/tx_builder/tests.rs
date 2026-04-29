use super::*;
use crate::stonfi::{model::SwapSimulation, testkit::TEST_TON_WALLET_ADDRESS};
use gem_ton::tvm::BagOfCells;

#[test]
fn test_build_v1_swap_transaction() {
    let v1: SwapSimulation = serde_json::from_str(include_str!("../testdata/v1_simulation.json")).unwrap();

    let v1_transaction = build_swap_transaction(SwapTransactionParams {
        simulation: &v1,
        from_native: true,
        to_native: false,
        from_value: "1000000000",
        min_ask_amount: &v1.min_ask_units,
        wallet_address: TEST_TON_WALLET_ADDRESS,
        receiver_address: TEST_TON_WALLET_ADDRESS,
        referral: ReferralParams {
            address: TEST_TON_WALLET_ADDRESS,
            bps: 50,
        },
        deadline: Some(1_700_000_000),
    })
    .unwrap();

    assert_eq!(v1_transaction.to, v1.offer_jetton_wallet);
    assert_eq!(v1_transaction.value, "1185000000");
    assert_eq!(
        v1_transaction.data,
        "te6cckEBAgEAqAABbQ+KfqUAAAAAAAAAAEO5rKAIAO87mQKicbKgHIk4pSPP4k5xhHqutqYgAB7USnesDnCcECwbgQMBANclk4VhgAndkkNzqarUGyjOwC2pOE1nNjryA0/Cp8zAZ+KNQRDehid7ywAM6FKWpQGl51ZuTKImFkWYLixc2NCsRYy79zJEauQV8/AAzoUpalAaXnVm5MoiYWRZguLFzY0KxFjLv3MkRq5BXz5OFmmt"
    );
    assert!(BagOfCells::parse_base64(&v1_transaction.data).is_ok());
}

#[test]
fn test_build_v2_swap_transactions() {
    let v2: SwapSimulation = serde_json::from_str(include_str!("../testdata/v2_simulation.json")).unwrap();

    let v2_transaction = build_swap_transaction(SwapTransactionParams {
        simulation: &v2,
        from_native: true,
        to_native: false,
        from_value: "1000000000",
        min_ask_amount: &v2.min_ask_units,
        wallet_address: TEST_TON_WALLET_ADDRESS,
        receiver_address: TEST_TON_WALLET_ADDRESS,
        referral: ReferralParams {
            address: TEST_TON_WALLET_ADDRESS,
            bps: 50,
        },
        deadline: Some(1_700_000_000),
    })
    .unwrap();

    assert_eq!(v2_transaction.to, v2.offer_jetton_wallet);
    assert_eq!(v2_transaction.value, "1310000000");
    assert_eq!(
        v2_transaction.data,
        "te6cckEBAwEA9QABZAHzg10AAAAAAAAAAEO5rKAIAGdClLUoDS86s3JlETCyLMFxYubGhWIsZd+5kiNXIK+fAQHhZmTeKoASRaxPr7HbegHJxHe2GKlO3cvD6MrnQ16ILwr/R8R9I/AAzoUpalAaXnVm5MoiYWRZguLFzY0KxFjLv3MkRq5BXz4AGdClLUoDS86s3JlETCyLMFxYubGhWIsZd+5kiNXIK+eAAAAAMqn4gEACAJMxNleIAGdClLUoDS86s3JlETCyLMFxYubGhWIsZd+5kiNXIK+eAAAZQAM6FKWpQGl51ZuTKImFkWYLixc2NCsRYy79zJEauQV8+IF8mPY="
    );
    assert!(BagOfCells::parse_base64(&v2_transaction.data).is_ok());

    let jetton_transaction = build_swap_transaction(SwapTransactionParams {
        simulation: &v2,
        from_native: false,
        to_native: true,
        from_value: "1000000",
        min_ask_amount: "740000000",
        wallet_address: TEST_TON_WALLET_ADDRESS,
        receiver_address: TEST_TON_WALLET_ADDRESS,
        referral: ReferralParams {
            address: TEST_TON_WALLET_ADDRESS,
            bps: 50,
        },
        deadline: Some(1_700_000_000),
    })
    .unwrap();

    assert_eq!(jetton_transaction.to, v2.offer_jetton_wallet);
    assert_eq!(jetton_transaction.value, "300000000");
    assert_eq!(
        jetton_transaction.data,
        "te6cckECAwEAARsAAa4Pin6lAAAAAAAAAAAw9CQIASXCgjXKjRJeZ2WRUT1SByGx/pn3ci9Mh3I85+4N+3OjAAzoUpalAaXnVm5MoiYWRZguLFzY0KxFjLv3MkRq5BXzyBycOAEBAeFmZN4qgBJFrE+vsdt6AcnEd7YYqU7dy8PoyudDXogvCv9HxH0j8ADOhSlqUBpedWbkyiJhZFmC4sXNjQrEWMu/cyRGrkFfPgAZ0KUtSgNLzqzcmURMLIswXFi5saFYixl37mSI1cgr54AAAAAyqfiAQAIAlULBuBAIAGdClLUoDS86s3JlETCyLMFxYubGhWIsZd+5kiNXIK+eAAAZQAM6FKWpQGl51ZuTKImFkWYLixc2NCsRYy79zJEauQV8+I2Pofs="
    );
    assert!(BagOfCells::parse_base64(&jetton_transaction.data).is_ok());
}
