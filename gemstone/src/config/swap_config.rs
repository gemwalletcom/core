#[derive(uniffi::Record, Debug, Clone, PartialEq)]
pub struct SwapConfig {
    pub default_slippage_bps: u32,
    pub referral_fee: SwapReferralFees,
}

#[derive(uniffi::Record, Default, Debug, Clone, PartialEq)]
pub struct SwapReferralFees {
    pub evm: SwapReferralFee,
    pub solana: SwapReferralFee,
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
            solana: SwapReferralFee::default(),
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
            solana: SwapReferralFee {
                address: "CK8n55Y664YjfifoVYfud8jXuSx9JV4NgVakEaRceVXu".into(),
                bps: 50,
            },
            thorchain: SwapReferralFee { address: "g1".into(), bps: 50 },
        },
    }
}
