pub const WALLET_ADDRESS: &str = "UQDxJKarPSp0bCta9DFgp81Mpt5hpGbuVcSxwfeza0Bin201";

pub fn v1_simulation() -> String {
    include_str!("v1_simulation.json").to_string()
}

pub fn v2_simulation() -> String {
    include_str!("v2_simulation.json").to_string()
}
