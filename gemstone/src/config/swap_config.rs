use crate::swapper::{GemSlippage, SlippageMode};
use primitives::Chain;

pub static DEFAULT_SLIPPAGE_BPS: u32 = 100;

#[derive(uniffi::Record, Debug, Clone, PartialEq)]
pub struct SwapConfig {
    pub default_slippage: GemSlippage,
    pub permit2_expiration: u64,
    pub permit2_sig_deadline: u64,
    pub referral_fee: SwapReferralFees,
}

#[derive(uniffi::Record, Default, Debug, Clone, PartialEq)]
pub struct SwapReferralFees {
    pub evm: SwapReferralFee,
    pub evm_bridge: SwapReferralFee,
    pub solana: SwapReferralFee,
    pub solana_jupiter: SwapReferralFee, // referral key
    pub thorchain: SwapReferralFee,
    pub sui_cetus: SwapReferralFee, // partner account
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
            sui_cetus: SwapReferralFee::default(),
        }
    }
}

pub fn get_swap_config() -> SwapConfig {
    SwapConfig {
        default_slippage: GemSlippage {
            bps: DEFAULT_SLIPPAGE_BPS,
            mode: SlippageMode::Exact,
        },
        permit2_expiration: 60 * 60 * 24 * 30, // 30 days
        permit2_sig_deadline: 60 * 30,         // 30 minutes
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
            sui_cetus: SwapReferralFee {
                address: "0x07754eecfffb541c38ff90c01d9e7881e1c6863374febb482ad7bf6dd129653c".into(),
                bps: 50,
            },
        },
    }
}

#[uniffi::export]
pub fn get_default_slippage(chain: &Chain) -> GemSlippage {
    match chain {
        Chain::Solana => GemSlippage {
            bps: DEFAULT_SLIPPAGE_BPS * 3,
            mode: SlippageMode::Auto,
        },
        _ => GemSlippage {
            bps: DEFAULT_SLIPPAGE_BPS,
            mode: SlippageMode::Exact,
        },
    }
}
