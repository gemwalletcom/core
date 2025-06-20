use super::{get_uniswap_permit2_by_chain, Deployment};
use primitives::Chain;

pub struct V3Deployment {
    pub quoter_v2: &'static str,
    pub permit2: &'static str,
    pub universal_router: &'static str,
}

impl Deployment for V3Deployment {
    fn quoter(&self) -> &'static str {
        self.quoter_v2
    }

    fn permit2(&self) -> &'static str {
        self.permit2
    }

    fn universal_router(&self) -> &'static str {
        self.universal_router
    }
}

pub fn get_uniswap_router_deployment_by_chain(chain: &Chain) -> Option<V3Deployment> {
    //https://docs.uniswap.org/contracts/v3/reference/deployments/
    let permit2 = get_uniswap_permit2_by_chain(chain)?;
    match chain {
        Chain::Ethereum => Some(V3Deployment {
            quoter_v2: "0x61fFE014bA17989E743c5F6cB21bF9697530B21e",
            permit2,
            universal_router: "0x3fC91A3afd70395Cd496C647d5a6CC9D4B2b7FAD",
        }),
        Chain::Optimism => Some(V3Deployment {
            quoter_v2: "0x61fFE014bA17989E743c5F6cB21bF9697530B21e",
            permit2,
            universal_router: "0xCb1355ff08Ab38bBCE60111F1bb2B784bE25D7e8",
        }),
        Chain::Arbitrum => Some(V3Deployment {
            quoter_v2: "0x61fFE014bA17989E743c5F6cB21bF9697530B21e",
            permit2,
            universal_router: "0x5E325eDA8064b456f4781070C0738d849c824258",
        }),
        Chain::Polygon => Some(V3Deployment {
            quoter_v2: "0x61fFE014bA17989E743c5F6cB21bF9697530B21e",
            permit2,
            universal_router: "0xec7BE89e9d109e7e3Fec59c222CF297125FEFda2",
        }),
        Chain::AvalancheC => Some(V3Deployment {
            quoter_v2: "0xbe0F5544EC67e9B3b2D979aaA43f18Fd87E6257F",
            permit2,
            universal_router: "0x4Dae2f939ACf50408e13d58534Ff8c2776d45265",
        }),
        Chain::Base => Some(V3Deployment {
            quoter_v2: "0x3d4e44Eb1374240CE5F1B871ab261CD16335B76a",
            permit2,
            universal_router: "0x3fC91A3afd70395Cd496C647d5a6CC9D4B2b7FAD",
        }),
        Chain::SmartChain => Some(V3Deployment {
            quoter_v2: "0x78D78E420Da98ad378D7799bE8f4AF69033EB077",
            permit2,
            universal_router: "0x4Dae2f939ACf50408e13d58534Ff8c2776d45265",
        }),
        Chain::ZkSync => Some(V3Deployment {
            quoter_v2: "0x8Cb537fc92E26d8EBBb760E632c95484b6Ea3e28",
            permit2,
            universal_router: "0x28731BCC616B5f51dD52CF2e4dF0E78dD1136C06",
        }),
        Chain::Celo => Some(V3Deployment {
            quoter_v2: "0x82825d0554fA07f7FC52Ab63c961F330fdEFa8E8",
            permit2,
            universal_router: "0x643770E279d5D0733F21d6DC03A8efbABf3255B4",
        }),
        Chain::Blast => Some(V3Deployment {
            quoter_v2: "0x6Cdcd65e03c1CEc3730AeeCd45bc140D57A25C77",
            permit2,
            universal_router: "0x643770E279d5D0733F21d6DC03A8efbABf3255B4",
        }),
        Chain::World => Some(V3Deployment {
            quoter_v2: "0x10158D43e6cc414deE1Bd1eB0EfC6a5cBCfF244c",
            permit2,
            universal_router: "0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D",
        }),

        Chain::Unichain => Some(V3Deployment {
            quoter_v2: "0x385A5cf5F83e99f7BB2852b6A19C3538b9FA7658",
            permit2,
            universal_router: "0xEf740bf23aCaE26f6492B10de645D6B98dC8Eaf3",
        }),
        _ => None,
    }
}

pub fn get_pancakeswap_router_deployment_by_chain(chain: &Chain) -> Option<V3Deployment> {
    // https://developer.pancakeswap.finance/contracts/universal-router/addresses
    // https://docs.pancakeswap.finance/developers/smart-contracts/pancakeswap-exchange/v3-contracts#address
    match chain {
        Chain::SmartChain => Some(V3Deployment {
            quoter_v2: "0xB048Bbc1Ee6b733FFfCFb9e9CeF7375518e25997",
            universal_router: "0x1A0A18AC4BECDDbd6389559687d1A73d8927E416",
            permit2: "0x31c2F6fcFf4F8759b3Bd5Bf0e1084A055615c768",
        }),
        Chain::OpBNB => Some(V3Deployment {
            quoter_v2: "0xB048Bbc1Ee6b733FFfCFb9e9CeF7375518e25997",
            universal_router: "0xB89a6778D1efE7a5b7096757A21b810CC2886fa1",
            permit2: "0x31c2F6fcFf4F8759b3Bd5Bf0e1084A055615c768",
        }),
        Chain::Arbitrum | Chain::Linea | Chain::Base => Some(V3Deployment {
            quoter_v2: "0xB048Bbc1Ee6b733FFfCFb9e9CeF7375518e25997",
            universal_router: "0xFE6508f0015C778Bdcc1fB5465bA5ebE224C9912",
            permit2: "0x31c2F6fcFf4F8759b3Bd5Bf0e1084A055615c768",
        }),
        _ => None,
    }
}

pub fn get_oku_deployment_by_chain(chain: &Chain) -> Option<V3Deployment> {
    // https://docs.oku.trade/home/extra-information/deployed-contracts
    match chain {
        Chain::Sonic => Some(V3Deployment {
            quoter_v2: "0x5911cB3633e764939edc2d92b7e1ad375Bb57649",
            universal_router: "0x738fD6d10bCc05c230388B4027CAd37f82fe2AF2",
            permit2: "0xB952578f3520EE8Ea45b7914994dcf4702cEe578",
        }),
        Chain::Mantle => Some(V3Deployment {
            quoter_v2: "0xdD489C75be1039ec7d843A6aC2Fd658350B067Cf",
            universal_router: "0x447B8E40B0CdA8e55F405C86bC635D02d0540aB8",
            permit2: "0x5d6b0f5335ec95cD2aB7E52f2A0750dd86502435",
        }),
        Chain::Gnosis => Some(V3Deployment {
            quoter_v2: "0x7E9cB3499A6cee3baBe5c8a3D328EA7FD36578f4",
            universal_router: "0x75FC67473A91335B5b8F8821277262a13B38c9b3",
            permit2: "0x000000000022D473030F116dDEE9F6B43aC78BA3",
        }),
        _ => None,
    }
}

pub fn get_wagmi_router_deployment_by_chain(chain: &Chain) -> Option<V3Deployment> {
    // https://docs.wagmi.com/wagmi/contracts#sonic
    match chain {
        Chain::Sonic => Some(V3Deployment {
            quoter_v2: "0xDb51CffFf3B989d0cB6b58AbF173371b6F2d0D24",
            universal_router: "0xC81dAe2Cdf2f6C0076aE3E174a54985040626D19",
            permit2: "0x7Ac9E324c2a211a389fac64b773433A17dB22948",
        }),
        _ => None,
    }
}

pub fn get_reservoir_deployment_by_chain(chain: &Chain) -> Option<V3Deployment> {
    // https://docs.reservoir.tools/reservoir-swap/uniswap-contracts
    let permit2 = get_uniswap_permit2_by_chain(chain)?;
    match chain {
        Chain::Abstract => Some(V3Deployment {
            quoter_v2: "0x728BD3eC25D5EDBafebB84F3d67367Cd9EBC7693",
            permit2,
            universal_router: "0xE1b076ea612Db28a0d768660e4D81346c02ED75e",
        }),
        Chain::Ink => Some(V3Deployment {
            quoter_v2: "0x96b572D2d880cf2Fa2563651BD23ADE6f5516652",
            permit2,
            universal_router: "0x9C5577aEF7c2a5C80aA62bA5420170F6b4a302FF",
        }),
        _ => None,
    }
}

pub fn get_aerodrome_router_deployment_by_chain(chain: &Chain) -> Option<V3Deployment> {
    // https://aerodrome.finance/security
    match chain {
        Chain::Base => Some(V3Deployment {
            quoter_v2: "0x254cF9E1E6e233aa1AC962CB9B05b2cfeAaE15b0",
            universal_router: "0x6Cb442acF35158D5eDa88fe602221b67B400Be3E",
            permit2: "0x0000000000000000000000000000000000000000", // Aerodrome does not use permit2
        }),
        _ => None,
    }
}
