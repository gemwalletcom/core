use super::*;
use crate::stonfi::{
    model::SwapSimulation,
    test::{WALLET_ADDRESS, v1_simulation, v2_simulation},
};
use gem_ton::signer::cells::BagOfCells;

#[test]
fn test_build_v1_swap_transaction() {
    let v1_simulation = v1_simulation();
    let v1: SwapSimulation = serde_json::from_str(&v1_simulation).unwrap();

    let v1_tx = build_swap_transaction(SwapTransactionParams {
        simulation: &v1,
        from_native: true,
        to_native: false,
        from_value: "1000000000",
        min_ask_amount: &v1.min_ask_units,
        wallet_address: WALLET_ADDRESS,
        receiver_address: WALLET_ADDRESS,
        referral: ReferralParams { address: WALLET_ADDRESS, bps: 50 },
        deadline: Some(1_700_000_000),
    })
    .unwrap();

    assert_eq!(v1_tx.to, v1.offer_jetton_wallet);
    assert_eq!(v1_tx.value, "1185000000");
    assert_eq!(
        v1_tx.data,
        "te6cckEBAgEAqAABbQ+KfqUAAAAAAAAAAEO5rKAIAO87mQKicbKgHIk4pSPP4k5xhHqutqYgAB7USnesDnCcECwbgQMBANclk4VhgAndkkNzqarUGyjOwC2pOE1nNjryA0/Cp8zAZ+KNQRDehid7ywA8SSmqz0qdGwrWvQxYKfNTKbeYaRm7lXEscH3s2tAYp/ADxJKarPSp0bCta9DFgp81Mpt5hpGbuVcSxwfeza0Bin7nsuh0"
    );
    assert!(BagOfCells::parse_base64(&v1_tx.data).is_ok());
}

#[test]
fn test_build_v2_swap_transactions() {
    let v2_simulation = v2_simulation();
    let v2: SwapSimulation = serde_json::from_str(&v2_simulation).unwrap();

    let v2_tx = build_swap_transaction(SwapTransactionParams {
        simulation: &v2,
        from_native: true,
        to_native: false,
        from_value: "1000000000",
        min_ask_amount: &v2.min_ask_units,
        wallet_address: WALLET_ADDRESS,
        receiver_address: WALLET_ADDRESS,
        referral: ReferralParams { address: WALLET_ADDRESS, bps: 50 },
        deadline: Some(1_700_000_000),
    })
    .unwrap();

    assert_eq!(v2_tx.to, v2.offer_jetton_wallet);
    assert_eq!(v2_tx.value, "1310000000");
    assert_eq!(
        v2_tx.data,
        "te6cckEBAwEA9QABZAHzg10AAAAAAAAAAEO5rKAIAeJJTVZ6VOjYVrXoYsFPmplNvMNIzdyriWOD72bWgMU/AQHhZmTeKoASRaxPr7HbegHJxHe2GKlO3cvD6MrnQ16ILwr/R8R9I/ADxJKarPSp0bCta9DFgp81Mpt5hpGbuVcSxwfeza0Bin4AeJJTVZ6VOjYVrXoYsFPmplNvMNIzdyriWOD72bWgMU+AAAAAMqn4gEACAJMxNleIAeJJTVZ6VOjYVrXoYsFPmplNvMNIzdyriWOD72bWgMU+AAAZQA8SSmqz0qdGwrWvQxYKfNTKbeYaRm7lXEscH3s2tAYp+HwgLSI="
    );
    assert!(BagOfCells::parse_base64(&v2_tx.data).is_ok());

    let jetton_tx = build_swap_transaction(SwapTransactionParams {
        simulation: &v2,
        from_native: false,
        to_native: true,
        from_value: "1000000",
        min_ask_amount: "740000000",
        wallet_address: WALLET_ADDRESS,
        receiver_address: WALLET_ADDRESS,
        referral: ReferralParams { address: WALLET_ADDRESS, bps: 50 },
        deadline: Some(1_700_000_000),
    })
    .unwrap();

    assert_eq!(jetton_tx.to, v2.offer_jetton_wallet);
    assert_eq!(jetton_tx.value, "300000000");
    assert_eq!(
        jetton_tx.data,
        "te6cckECAwEAARsAAa4Pin6lAAAAAAAAAAAw9CQIASXCgjXKjRJeZ2WRUT1SByGx/pn3ci9Mh3I85+4N+3OjADxJKarPSp0bCta9DFgp81Mpt5hpGbuVcSxwfeza0BinyBycOAEBAeFmZN4qgBJFrE+vsdt6AcnEd7YYqU7dy8PoyudDXogvCv9HxH0j8APEkpqs9KnRsK1r0MWCnzUym3mGkZu5VxLHB97NrQGKfgB4klNVnpU6NhWtehiwU+amU28w0jN3KuJY4PvZtaAxT4AAAAAyqfiAQAIAlULBuBAIAeJJTVZ6VOjYVrXoYsFPmplNvMNIzdyriWOD72bWgMU+AAAZQA8SSmqz0qdGwrWvQxYKfNTKbeYaRm7lXEscH3s2tAYp+ObkKKY="
    );
    assert!(BagOfCells::parse_base64(&jetton_tx.data).is_ok());
}
