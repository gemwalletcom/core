use super::{DEFAULT_STABLE_SWAP_REFERRAL_BPS, DEFAULT_SWAP_FEE_BPS, EVM_REFERRAL_ADDRESS};

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
    pub aptos: ReferralFee,
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct ReferralFee {
    pub address: String,
    pub bps: u32,
}

impl ReferralFees {
    pub fn evm(evm: ReferralFee) -> Self {
        Self { evm, ..Default::default() }
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
            &mut self.aptos,
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

pub fn default_referral_fees() -> ReferralFees {
    ReferralFees {
        evm: ReferralFee {
            address: EVM_REFERRAL_ADDRESS.into(),
            bps: DEFAULT_SWAP_FEE_BPS,
        },
        evm_bridge: ReferralFee {
            address: EVM_REFERRAL_ADDRESS.into(),
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
        aptos: ReferralFee {
            address: "0xc09d385527743bb03ed7847bb9180b5ff2263d38d5a93f1c9b3068f8505f6488".into(),
            bps: DEFAULT_SWAP_FEE_BPS,
        },
    }
}
