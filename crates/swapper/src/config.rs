use crate::{SwapperSlippage, SwapperSlippageMode};
use primitives::Chain;

pub static DEFAULT_SLIPPAGE_BPS: u32 = 100;
pub static DEFAULT_SWAP_FEE_BPS: u32 = 50;
pub static DEFAULT_CHAINFLIP_FEE_BPS: u32 = 45;
pub static DEFAULT_STABLE_SWAP_REFERRAL_BPS: u32 = 25;

pub const API_BASE_URL: &str = "https://api.gemwallet.com/v1/swap";

pub fn get_swap_api_url(path: &str) -> String {
    format!("{API_BASE_URL}/{path}")
}

#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    pub default_slippage: SwapperSlippage,
    pub permit2_expiration: u64,
    pub permit2_sig_deadline: u64,
    pub referral_fee: ReferralFees,
    pub high_price_impact_percent: u32,
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct ReferralFees {
    pub evm: ReferralFee,
    pub evm_bridge: ReferralFee,
    pub solana: ReferralFee,
    pub thorchain: ReferralFee,
    pub sui: ReferralFee,
    pub ton: ReferralFee,
    pub tron: ReferralFee,
    pub near: ReferralFee,
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct ReferralFee {
    pub address: String,
    pub bps: u32,
}

impl ReferralFees {
    pub fn evm(evm: ReferralFee) -> ReferralFees {
        ReferralFees {
            evm,
            evm_bridge: ReferralFee::default(),
            solana: ReferralFee::default(),
            thorchain: ReferralFee::default(),
            sui: ReferralFee::default(),
            ton: ReferralFee::default(),
            tron: ReferralFee::default(),
            near: ReferralFee::default(),
        }
    }

    pub fn update_all_bps(&mut self, bps: u32) {
        self.iter_mut().for_each(|fee| fee.update_bps(bps));
    }

    fn iter_mut(&mut self) -> impl Iterator<Item = &mut ReferralFee> {
        [
            &mut self.evm,
            &mut self.evm_bridge,
            &mut self.solana,
            &mut self.thorchain,
            &mut self.sui,
            &mut self.ton,
            &mut self.tron,
            &mut self.near,
        ]
        .into_iter()
    }
}

impl ReferralFee {
    pub fn update_bps(&mut self, bps: u32) {
        if !self.address.is_empty() || self.bps > 0 {
            self.bps = bps;
        }
    }
}

pub fn get_swap_config() -> Config {
    Config {
        default_slippage: SwapperSlippage {
            bps: DEFAULT_SLIPPAGE_BPS,
            mode: SwapperSlippageMode::Exact,
        },
        permit2_expiration: 60 * 60 * 24 * 30, // 30 days
        permit2_sig_deadline: 60 * 30,         // 30 minutes
        referral_fee: ReferralFees {
            evm: ReferralFee {
                address: "0x0D9DAB1A248f63B0a48965bA8435e4de7497a3dC".into(),
                bps: DEFAULT_SWAP_FEE_BPS,
            },
            evm_bridge: ReferralFee {
                address: "0x0D9DAB1A248f63B0a48965bA8435e4de7497a3dC".into(),
                bps: DEFAULT_STABLE_SWAP_REFERRAL_BPS,
            },
            solana: ReferralFee {
                address: "5fmLrs2GuhfDP1B51ziV5Kd1xtAr9rw1jf3aQ4ihZ2gy".into(),
                bps: DEFAULT_SWAP_FEE_BPS,
            },
            thorchain: ReferralFee {
                address: "g1".into(),
                bps: DEFAULT_SWAP_FEE_BPS,
            },
            sui: ReferralFee {
                address: "0x9d6b98b18fd26b5efeec68d020dcf1be7a94c2c315353779bc6b3aed44188ddf".into(),
                bps: DEFAULT_SWAP_FEE_BPS,
            },
            ton: ReferralFee {
                address: "UQDxJKarPSp0bCta9DFgp81Mpt5hpGbuVcSxwfeza0Bin201".into(),
                bps: DEFAULT_SWAP_FEE_BPS,
            },
            tron: ReferralFee {
                address: "TYeyZXywpA921LEtw2PF3obK4B8Jjgpp32".into(),
                bps: DEFAULT_SWAP_FEE_BPS,
            },
            near: ReferralFee {
                address: "0x0d9dab1a248f63b0a48965ba8435e4de7497a3dc".into(),
                bps: DEFAULT_SWAP_FEE_BPS,
            },
        },
        high_price_impact_percent: 10,
    }
}

pub fn get_default_slippage(chain: &Chain) -> SwapperSlippage {
    match chain {
        Chain::Solana => SwapperSlippage {
            bps: DEFAULT_SLIPPAGE_BPS * 3,
            mode: SwapperSlippageMode::Auto,
        },
        _ => SwapperSlippage {
            bps: DEFAULT_SLIPPAGE_BPS,
            mode: SwapperSlippageMode::Exact,
        },
    }
}
