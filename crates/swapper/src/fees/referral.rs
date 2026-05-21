use super::DEFAULT_SWAP_FEE_BPS;
use primitives::{Chain, ChainType};

#[derive(Default, Debug, Clone, PartialEq)]
pub struct ReferralFees {
    pub evm: ReferralFee,
    pub solana: ReferralFee,
    pub thorchain: ReferralFee,
    pub sui: ReferralFee,
    pub ton: ReferralFee,
    pub tron: ReferralFee,
    pub near: ReferralFee,
    pub aptos: ReferralFee,
    pub cosmos: ReferralFee,
    pub injective: ReferralFee,
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

    pub fn for_chain(&self, chain: Chain) -> Option<&ReferralFee> {
        let fee = match chain.chain_type() {
            ChainType::Ethereum => &self.evm,
            ChainType::Solana => &self.solana,
            ChainType::Sui => &self.sui,
            ChainType::Ton => &self.ton,
            ChainType::Tron => &self.tron,
            ChainType::Near => &self.near,
            ChainType::Aptos => &self.aptos,
            ChainType::Cosmos => match chain {
                Chain::Thorchain => &self.thorchain,
                Chain::Injective => &self.injective,
                _ => &self.cosmos,
            },
            ChainType::Bitcoin | ChainType::Xrp | ChainType::Stellar | ChainType::Algorand | ChainType::Polkadot | ChainType::Cardano | ChainType::HyperCore => return None,
        };
        Some(fee)
    }

    pub fn bps_for_chain(&self, chain: Chain) -> u32 {
        self.for_chain(chain).map(|fee| fee.bps).unwrap_or(0)
    }
}

pub fn default_referral_fees() -> ReferralFees {
    ReferralFees {
        evm: ReferralFee {
            address: "0x0D9DAB1A248f63B0a48965bA8435e4de7497a3dC".into(),
            bps: DEFAULT_SWAP_FEE_BPS,
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
        cosmos: ReferralFee {
            address: "cosmos1knwywgnzs3a2p39k7337klt6daqrhyvnh8vz27".into(),
            bps: DEFAULT_SWAP_FEE_BPS,
        },
        injective: ReferralFee {
            address: "inj1pkw6kx3y3a3mpfyfvkaggd0yme6f0g7uylvm5y".into(),
            bps: DEFAULT_SWAP_FEE_BPS,
        },
    }
}

fn default_referral_fee(chain: Chain) -> ReferralFee {
    default_referral_fees().for_chain(chain).cloned().unwrap_or_default()
}

pub fn default_referral_address(chain: Chain) -> String {
    default_referral_fee(chain).address
}
