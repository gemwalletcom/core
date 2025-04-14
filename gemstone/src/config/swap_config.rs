use crate::swapper::{GemSlippage, GemSlippageMode};
use primitives::Chain;

pub static DEFAULT_SLIPPAGE_BPS: u32 = 100;
pub static DEFAULT_SWAP_FEE_BPS: u32 = 50;
pub static DEFAULT_BRIDGE_FEE_BPS: u32 = 25;

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
    pub sui: SwapReferralFee,
    pub ton: SwapReferralFee,
    pub tron: SwapReferralFee,
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
            sui: SwapReferralFee::default(),
            ton: SwapReferralFee::default(),
            tron: SwapReferralFee::default(),
        }
    }
}

pub fn get_swap_config() -> SwapConfig {
    SwapConfig {
        default_slippage: GemSlippage {
            bps: DEFAULT_SLIPPAGE_BPS,
            mode: GemSlippageMode::Exact,
        },
        permit2_expiration: 60 * 60 * 24 * 30, // 30 days
        permit2_sig_deadline: 60 * 30,         // 30 minutes
        referral_fee: SwapReferralFees {
            evm: SwapReferralFee {
                address: "0x0D9DAB1A248f63B0a48965bA8435e4de7497a3dC".into(),
                bps: DEFAULT_SWAP_FEE_BPS,
            },
            evm_bridge: SwapReferralFee {
                address: "0x0D9DAB1A248f63B0a48965bA8435e4de7497a3dC".into(),
                bps: DEFAULT_BRIDGE_FEE_BPS,
            },
            solana: SwapReferralFee {
                address: "5fmLrs2GuhfDP1B51ziV5Kd1xtAr9rw1jf3aQ4ihZ2gy".into(),
                bps: DEFAULT_SWAP_FEE_BPS,
            },
            solana_jupiter: SwapReferralFee {
                address: "CK8n55Y664YjfifoVYfud8jXuSx9JV4NgVakEaRceVXu".into(),
                bps: DEFAULT_SWAP_FEE_BPS,
            },
            thorchain: SwapReferralFee {
                address: "g1".into(),
                bps: DEFAULT_SWAP_FEE_BPS,
            },
            sui: SwapReferralFee {
                address: "0x9d6b98b18fd26b5efeec68d020dcf1be7a94c2c315353779bc6b3aed44188ddf".into(),
                bps: DEFAULT_SWAP_FEE_BPS,
            },
            ton: SwapReferralFee {
                address: "UQDxJKarPSp0bCta9DFgp81Mpt5hpGbuVcSxwfeza0Bin201".into(),
                bps: DEFAULT_SWAP_FEE_BPS,
            },
            tron: SwapReferralFee {
                address: "TA7mCjHFfo68FG3wc6pDCeRGbJSPZkBfL7".into(),
                bps: DEFAULT_SWAP_FEE_BPS,
            },
        },
    }
}

#[uniffi::export]
pub fn get_default_slippage(chain: &Chain) -> GemSlippage {
    match chain {
        Chain::Solana => GemSlippage {
            bps: DEFAULT_SLIPPAGE_BPS * 3,
            mode: GemSlippageMode::Auto,
        },
        _ => GemSlippage {
            bps: DEFAULT_SLIPPAGE_BPS,
            mode: GemSlippageMode::Exact,
        },
    }
}
