use super::{Deployment, get_uniswap_permit2_by_chain};
use primitives::Chain;

pub struct V4Deployment {
    pub quoter: &'static str, // V4 Quoter
    pub permit2: &'static str,
    pub universal_router: &'static str,
}

impl Deployment for V4Deployment {
    fn quoter(&self) -> &'static str {
        self.quoter
    }

    fn permit2(&self) -> &'static str {
        self.permit2
    }

    fn universal_router(&self) -> &'static str {
        self.universal_router
    }
}

pub fn get_uniswap_deployment_by_chain(chain: &Chain) -> Option<V4Deployment> {
    // https://github.com/Uniswap/contracts/blob/main/deployments/index.md
    let permit2 = get_uniswap_permit2_by_chain(chain)?;
    match chain {
        Chain::Ethereum => Some(V4Deployment {
            quoter: "0x52f0e24d1c21c8a0cb1e5a5dd6198556bd9e1203",
            permit2,
            universal_router: "0x66a9893cc07d91d95644aedd05d03f95e1dba8af",
        }),
        Chain::Optimism => Some(V4Deployment {
            quoter: "0x1f3131a13296fb91c90870043742c3cdbff1a8d7",
            permit2,
            universal_router: "0x851116d9223fabed8e56c0e6b8ad0c31d98b3507",
        }),
        Chain::Arbitrum => Some(V4Deployment {
            quoter: "0x3972c00f7ed4885e145823eb7c655375d275a1c5",
            permit2,
            universal_router: "0xa51afafe0263b40edaef0df8781ea9aa03e381a3",
        }),
        Chain::Polygon => Some(V4Deployment {
            quoter: "0xb3d5c3dfc3a7aebff71895a7191796bffc2c81b9",
            permit2,
            universal_router: "0x1095692a6237d83c6a72f3f5efedb9a670c49223",
        }),
        Chain::AvalancheC => Some(V4Deployment {
            quoter: "0xbe40675bb704506a3c2ccfb762dcfd1e979845c2",
            permit2,
            universal_router: "0x94b75331ae8d42c1b61065089b7d48fe14aa73b7",
        }),
        Chain::Base => Some(V4Deployment {
            quoter: "0x0d5e0f971ed27fbff6c2837bf31316121532048d",
            permit2,
            universal_router: "0x6ff5693b99212da76ad316178a184ab56d299b43",
        }),
        Chain::SmartChain => Some(V4Deployment {
            quoter: "0x9f75dd27d6664c475b90e105573e550ff69437b0",
            permit2,
            universal_router: "0x1906c1d672b88cd1b9ac7593301ca990f94eae07",
        }),
        Chain::Blast => Some(V4Deployment {
            quoter: "0x6f71cdcb0d119ff72c6eb501abceb576fbf62bcf",
            permit2,
            universal_router: "0xeabbcb3e8e415306207ef514f660a3f820025be3",
        }),
        Chain::World => Some(V4Deployment {
            quoter: "0x55d235b3ff2daf7c3ede0defc9521f1d6fe6c5c0",
            permit2,
            universal_router: "0x8ac7bee993bb44dab564ea4bc9ea67bf9eb5e743",
        }),
        Chain::Unichain => Some(V4Deployment {
            quoter: "0x333E3C607B141b18fF6de9f258db6e77fE7491E0",
            permit2,
            universal_router: "0xEf740bf23aCaE26f6492B10de645D6B98dC8Eaf3",
        }),
        _ => None,
    }
}
