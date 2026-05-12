pub use primitives::testkit::signer_mock::TEST_TON_SENDER;

use super::{
    model::{Router, SwapSimulation},
    tx_builder::{ReferralParams, SwapTransactionParams},
};
use gem_ton::address::Address;

#[cfg(all(test, feature = "swap_integration_tests"))]
pub const NOT_TOKEN_ID: &str = "EQAvlWFDxGF2lXm67y4yzC17wYKD9A0guwPkMs1gOsM__NOT";
pub const ROUTER_V2_ADDRESS: &str = "EQCS4UEa5UaJLzOyyKieqQOQ2P9M-7kXpkO5HnP3Bv250cN3";

impl SwapSimulation {
    pub fn mock(offer_jetton_wallet: &str, ask_jetton_wallet: &str, ask_units: &str, min_ask_units: &str) -> Self {
        Self {
            offer_jetton_wallet: offer_jetton_wallet.to_string(),
            ask_jetton_wallet: ask_jetton_wallet.to_string(),
            router: Router {
                address: ROUTER_V2_ADDRESS.to_string(),
                major_version: 2,
                minor_version: 2,
            },
            ask_units: ask_units.to_string(),
            min_ask_units: min_ask_units.to_string(),
        }
    }
}

impl<'a> SwapTransactionParams<'a> {
    pub fn mock(simulation: &'a SwapSimulation) -> Self {
        Self {
            simulation,
            next_swap: None,
            from_native: true,
            to_native: false,
            sender_jetton_wallet: None,
            from_value: "1000000000",
            wallet_address: Address::parse(TEST_TON_SENDER).unwrap(),
            receiver_address: Address::parse(TEST_TON_SENDER).unwrap(),
            referral: ReferralParams {
                address: Address::parse(TEST_TON_SENDER).unwrap(),
                bps: 50,
            },
            deadline: Some(1_700_000_000),
        }
    }
}
