use super::models::FeeTier;

pub fn get_all_fee_tiers() -> Vec<FeeTier> {
    let fee_tiers = vec![
        FeeTier::new(1, 100),
        FeeTier::new(2, 200),
        FeeTier::new(4, 400),
        FeeTier::new(8, 500),
        FeeTier::new(16, 1600),
        FeeTier::new(64, 3000),
        FeeTier::new(96, 6500),
        FeeTier::new(128, 10000),
        FeeTier::new(32896, 10000),
        FeeTier::new(256, 20000),
    ];
    fee_tiers
}

pub fn get_splash_pool_fee_tiers() -> FeeTier {
    FeeTier::new(32896, 10000)
}
