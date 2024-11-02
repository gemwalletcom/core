use primitives::Chain;

#[allow(dead_code)]
pub struct V3Deployment {
    pub quoter_v2: &'static str,
    pub permit2: &'static str,
    pub universal_router: &'static str,
}

pub fn get_deployment_by_chain(chain: Chain) -> Option<V3Deployment> {
    // https://docs.uniswap.org/contracts/v3/reference/deployments/
    match chain {
        Chain::Ethereum => Some(V3Deployment {
            quoter_v2: "0x61fFE014bA17989E743c5F6cB21bF9697530B21e",
            permit2: "0x000000000022D473030F116dDEE9F6B43aC78BA3",
            universal_router: "0x3fC91A3afd70395Cd496C647d5a6CC9D4B2b7FAD",
        }),
        Chain::Optimism => Some(V3Deployment {
            quoter_v2: "0x61fFE014bA17989E743c5F6cB21bF9697530B21e",
            permit2: "0x000000000022D473030F116dDEE9F6B43aC78BA3",
            universal_router: "0xCb1355ff08Ab38bBCE60111F1bb2B784bE25D7e8",
        }),
        Chain::Arbitrum => Some(V3Deployment {
            quoter_v2: "0x61fFE014bA17989E743c5F6cB21bF9697530B21e",
            permit2: "0x000000000022D473030F116dDEE9F6B43aC78BA3",
            universal_router: "0x5E325eDA8064b456f4781070C0738d849c824258",
        }),
        Chain::Polygon => Some(V3Deployment {
            quoter_v2: "0x61fFE014bA17989E743c5F6cB21bF9697530B21e",
            permit2: "0x000000000022D473030F116dDEE9F6B43aC78BA3",
            universal_router: "0xec7BE89e9d109e7e3Fec59c222CF297125FEFda2",
        }),
        Chain::AvalancheC => Some(V3Deployment {
            quoter_v2: "0xbe0F5544EC67e9B3b2D979aaA43f18Fd87E6257F",
            permit2: "0x000000000022D473030F116dDEE9F6B43aC78BA3",
            universal_router: "0x4Dae2f939ACf50408e13d58534Ff8c2776d45265",
        }),
        Chain::Base => Some(V3Deployment {
            quoter_v2: "0x3d4e44Eb1374240CE5F1B871ab261CD16335B76a",
            permit2: "0x000000000022D473030F116dDEE9F6B43aC78BA3",
            universal_router: "0x3fC91A3afd70395Cd496C647d5a6CC9D4B2b7FAD",
        }),
        Chain::SmartChain => Some(V3Deployment {
            quoter_v2: "0x78D78E420Da98ad378D7799bE8f4AF69033EB077",
            permit2: "0x000000000022D473030F116dDEE9F6B43aC78BA3",
            universal_router: "0x4Dae2f939ACf50408e13d58534Ff8c2776d45265",
        }),
        Chain::ZkSync => Some(V3Deployment {
            quoter_v2: "0x8Cb537fc92E26d8EBBb760E632c95484b6Ea3e28",
            permit2: "0x0000000000225e31d15943971f47ad3022f714fa",
            universal_router: "0x28731BCC616B5f51dD52CF2e4dF0E78dD1136C06",
        }),
        Chain::Celo => Some(V3Deployment {
            quoter_v2: "0x82825d0554fA07f7FC52Ab63c961F330fdEFa8E8",
            permit2: "0x000000000022D473030F116dDEE9F6B43aC78BA3",
            universal_router: "0x643770E279d5D0733F21d6DC03A8efbABf3255B4",
        }),
        Chain::Blast => Some(V3Deployment {
            quoter_v2: "0x6Cdcd65e03c1CEc3730AeeCd45bc140D57A25C77",
            permit2: "0x000000000022d473030f116ddee9f6b43ac78ba3",
            universal_router: "0x643770E279d5D0733F21d6DC03A8efbABf3255B4",
        }),
        Chain::World => Some(V3Deployment {
            quoter_v2: "0x10158D43e6cc414deE1Bd1eB0EfC6a5cBCfF244c",
            permit2: "0x000000000022d473030f116ddee9f6b43ac78ba3",
            universal_router: "0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D",
        }),
        _ => None,
    }
}
