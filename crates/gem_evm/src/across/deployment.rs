use primitives::Chain;

pub struct AcrossDeployment {
    pub chain_id: u32,
    pub hub_pool: &'static str, // only for mainnet
    pub spoke_pool: &'static str,
}

impl AcrossDeployment {
    pub fn deployed_chains() -> Vec<Chain> {
        vec![
            Chain::Ethereum,
            Chain::Arbitrum,
            Chain::Base,
            Chain::Blast,
            Chain::Linea,
            Chain::Optimism,
            Chain::Polygon,
            Chain::World,
            Chain::ZkSync,
        ]
    }
    pub fn deployment_by_chain(chain: &Chain) -> Option<Self> {
        match chain {
            Chain::Ethereum => Some(Self {
                chain_id: 1,
                hub_pool: "0xc186fA914353c44b2E33eBE05f21846F1048bEda",
                spoke_pool: "0x5c7BCd6E7De5423a257D81B442095A1a6ced35C5",
            }),
            Chain::Arbitrum => Some(Self {
                chain_id: 42161,
                hub_pool: "",
                spoke_pool: "0xe35e9842fceaca96570b734083f4a58e8f7c5f2a",
            }),
            Chain::Base => Some(Self {
                chain_id: 8453,
                hub_pool: "",
                spoke_pool: "0x09aea4b2242abC8bb4BB78D537A67a245A7bEC64",
            }),
            Chain::Blast => Some(Self {
                chain_id: 81457,
                hub_pool: "",
                spoke_pool: "0x2D509190Ed0172ba588407D4c2df918F955Cc6E1",
            }),
            Chain::Linea => Some(Self {
                chain_id: 59144,
                hub_pool: "",
                spoke_pool: "0x7E63A5f1a8F0B4d0934B2f2327DAED3F6bb2ee75",
            }),
            Chain::Optimism => Some(Self {
                chain_id: 10,
                hub_pool: "",
                spoke_pool: "0x6f26Bf09B1C792e3228e5467807a900A503c0281",
            }),
            Chain::Polygon => Some(Self {
                chain_id: 137,
                hub_pool: "",
                spoke_pool: "0x9295ee1d8C5b022Be115A2AD3c30C72E34e7F096",
            }),
            Chain::World => Some(Self {
                chain_id: 480,
                hub_pool: "",
                spoke_pool: "0x09aea4b2242abC8bb4BB78D537A67a245A7bEC64",
            }),
            Chain::ZkSync => Some(Self {
                chain_id: 324,
                hub_pool: "",
                spoke_pool: "0xE0B015E54d54fc84a6cB9B666099c46adE9335FF",
            }),
            _ => None,
        }
    }
}
