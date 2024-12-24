#[derive(uniffi::Record, Debug, Clone, PartialEq)]
pub struct SwapConfig {
    pub default_slippage_bps: u32,
    pub referral_fee: SwapReferralFees,
}

#[derive(uniffi::Record, Default, Debug, Clone, PartialEq)]
pub struct SwapReferralFees {
    pub evm: SwapReferralFee,
    pub evm_bridge: SwapReferralFee,
    pub solana: SwapReferralFee,
    pub solana_jupiter: SwapReferralFee,
    pub thorchain: SwapReferralFee,
}

#[derive(uniffi::Record, Default, Debug, Clone, PartialEq)]
pub struct SwapReferralFee {
    pub address: String,
    pub bps: u32,
}

impl SwapReferralFees {
    pub fn evm(evm: SwapReferralFee) -> SwapReferralFees {
        SwapReferralFees {
            evm,
            evm_bridge: SwapReferralFee::default(),
            solana: SwapReferralFee::default(),
            solana_jupiter: SwapReferralFee::default(),
            thorchain: SwapReferralFee::default(),
        }
    }
}

pub fn get_swap_config() -> SwapConfig {
    SwapConfig {
        default_slippage_bps: 100,
        referral_fee: SwapReferralFees {
            evm: SwapReferralFee {
                address: "0x0D9DAB1A248f63B0a48965bA8435e4de7497a3dC".into(),
                bps: 50,
            },
            evm_bridge: SwapReferralFee {
                address: "0x0D9DAB1A248f63B0a48965bA8435e4de7497a3dC".into(),
                bps: 25,
            },
            solana: SwapReferralFee {
                address: "5fmLrs2GuhfDP1B51ziV5Kd1xtAr9rw1jf3aQ4ihZ2gy".into(),
                bps: 50,
            },
            solana_jupiter: SwapReferralFee {
                address: "CK8n55Y664YjfifoVYfud8jXuSx9JV4NgVakEaRceVXu".into(),
                bps: 50,
            },
            thorchain: SwapReferralFee { address: "g1".into(), bps: 50 },
        },
    }
}
