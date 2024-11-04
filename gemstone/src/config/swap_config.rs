#[derive(uniffi::Record, Debug, Clone, PartialEq)]
pub struct SwapConfig {
    slippage_bps: u32,
    referral_fee: SwapReferralFees,
}

#[derive(uniffi::Record, Debug, Clone, PartialEq)]
pub struct SwapReferralFees {
    evm: SwapReferralFee,
    solana: SwapReferralFee,
    thorchain: SwapReferralFee,
}

#[derive(uniffi::Record, Debug, Clone, PartialEq)]
pub struct SwapReferralFee {
    address: String,
    bps: u32,
}

pub fn get_swap_config() -> SwapConfig {
    SwapConfig {
        slippage_bps: 100,
        referral_fee: SwapReferralFees {
            evm: SwapReferralFee { address: "0x0D9DAB1A248f63B0a48965bA8435e4de7497a3dC".into(), bps: 50 },
            solana: SwapReferralFee { address: "5fmLrs2GuhfDP1B51ziV5Kd1xtAr9rw1jf3aQ4ihZ2gy".into(), bps: 50 },
            thorchain: SwapReferralFee { address: "gemwallet".into(), bps: 50 },
        },
    }
}