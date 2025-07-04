use alloy_primitives::{address, Address};
use primitives::Chain;

#[derive(Debug, Clone)]
pub struct ContractEntry {
    pub address: Address,
    pub provider: &'static str,
    pub chain: Chain,
}

#[derive(Debug, Clone)]
pub struct ContractRegistry {
    pub entries: Vec<ContractEntry>,
}

impl ContractRegistry {
    pub fn new() -> Self {
        let entries = vec![
            ContractEntry {
                address: address!("0x5968feacba91d55010975e0cfe8acfc32664ad33"),
                provider: "PancakeSwap v3",
                chain: Chain::SmartChain,
            },
            ContractEntry {
                address: address!("0x380aadf63d84d3a434073f1d5d95f02fb23d5228"),
                provider: "PancakeSwap v3",
                chain: Chain::SmartChain,
            },
            ContractEntry {
                address: address!("0x111111125421ca6dc452d289314280a0f8842a65"),
                provider: "1inch v6",
                chain: Chain::SmartChain,
            },
            ContractEntry {
                address: address!("0x099f84de4fb511e861ca8f635623eae409405873"),
                provider: "PancakeSwap v3",
                chain: Chain::SmartChain,
            },
            ContractEntry {
                address: address!("0x882df4b0fb50a229c3b4124eb18c759911485bfb"),
                provider: "QuickSwap v2",
                chain: Chain::Polygon,
            },
            ContractEntry {
                address: address!("0x172fcd41e0913e95784454622d1c3724f546f849"),
                provider: "PancakeSwap v3",
                chain: Chain::SmartChain,
            },
            ContractEntry {
                address: address!("0x7d94b911a51670f78a44a7af3c2bf773c42f2497"),
                provider: "PancakeSwap v3",
                chain: Chain::SmartChain,
            },
            ContractEntry {
                address: address!("0x08a10ae012df633abbf710ef8bd3a9745a9e5816"),
                provider: "PancakeSwap v3",
                chain: Chain::SmartChain,
            },
            ContractEntry {
                address: address!("0xcf59b8c8baa2dea520e3d549f97d4e49ade17057"),
                provider: "PancakeSwap v3",
                chain: Chain::SmartChain,
            },
            ContractEntry {
                address: address!("0x28e2ea090877bf75740558f6bfb36a5ffee9e9df"),
                provider: "Uniswap v4",
                chain: Chain::SmartChain,
            },
            ContractEntry {
                address: address!("0xd7af60112d7dfe0f914724e3407dd54424aaa19b"),
                provider: "PancakeSwap v3",
                chain: Chain::SmartChain,
            },
            ContractEntry {
                address: address!("0x498581ff718922c3f8e6a244956af099b2652b2b"),
                provider: "Uniswap v4",
                chain: Chain::Base,
            },
            ContractEntry {
                address: address!("0xc1a780989734a0e5df875cebe410748562e1c5e6"),
                provider: "PancakeSwap v3",
                chain: Chain::SmartChain,
            },
            ContractEntry {
                address: address!("0x1f98400000000000000000000000000000000004"),
                provider: "Uniswap v4",
                chain: Chain::Unichain,
            },
            ContractEntry {
                address: address!("0x656840f632cab4757f25a56d42fac9f51e3f49a2"),
                provider: "0x Protocol",
                chain: Chain::World,
            },
            ContractEntry {
                address: address!("0x72ab388e2e2f6facef59e3c3fa2c4e29011c2d38"),
                provider: "PancakeSwap v3",
                chain: Chain::Base,
            },
            ContractEntry {
                address: address!("0xd17a8609b5d95a5f49b290c4d787949bfec5279e"),
                provider: "Uniswap v2",
                chain: Chain::Base,
            },
            ContractEntry {
                address: address!("0xcaf2da315f5a5499299a312b8a86faafe4bad959"),
                provider: "0x Protocol",
                chain: Chain::Base,
            },
            ContractEntry {
                address: address!("0xf2688fb5b81049dfb7703ada5e770543770612c4"),
                provider: "PancakeSwap v3",
                chain: Chain::SmartChain,
            },
            ContractEntry {
                address: address!("0x36696169c63e42cd08ce11f5deebbcebae652050"),
                provider: "PancakeSwap v3",
                chain: Chain::SmartChain,
            },
            ContractEntry {
                address: address!("0xea27b3e61144f0417f27aedaa1b9e46fa5a49ff1"),
                provider: "PancakeSwap v3",
                chain: Chain::SmartChain,
            },
            ContractEntry {
                address: address!("0x47a90a2d92a8367a91efa1906bfc8c1e05bf10c4"),
                provider: "Uniswap v3",
                chain: Chain::SmartChain,
            },
            ContractEntry {
                address: address!("0xbef8358ab02b1af3b9d8af97e8963e9ca4f92727"),
                provider: "SyncSwap v2",
                chain: Chain::Ethereum,
            },
            ContractEntry {
                address: address!("0x6f38e884725a116c9c7fbf208e79fe8828a2595f"),
                provider: "Uniswap v3",
                chain: Chain::Arbitrum,
            },
            ContractEntry {
                address: address!("0x16b9a82891338f9ba80e2d6970fdda79d1eb0dae"),
                provider: "PancakeSwap v2",
                chain: Chain::SmartChain,
            },
            ContractEntry {
                address: address!("0x7fcdc35463e3770c2fb992716cd070b63540b947"),
                provider: "PancakeSwap v3",
                chain: Chain::Arbitrum,
            },
            ContractEntry {
                address: address!("0x69b86059c5fb3a44355937e7b505a659443b9a22"),
                provider: "PancakeSwap v3",
                chain: Chain::SmartChain,
            },
            ContractEntry {
                address: address!("0xb1026b8e7276e7ac75410f1fcbbe21796e8f7526"),
                provider: "Camelot v3",
                chain: Chain::Arbitrum,
            },
            ContractEntry {
                address: address!("0x6131b5fae19ea4f9d964eac0408e4408b66337b5"),
                provider: "KyberSwap Meta v2",
                chain: Chain::Base,
            },
            ContractEntry {
                address: address!("0x0ea1f3adb8fa795d64d39beccb7c36f8aed455f3"),
                provider: "0x Protocol",
                chain: Chain::World,
            },
            ContractEntry {
                address: address!("0x111111125421ca6dc452d289314280a0f8842a65"),
                provider: "1inch v6",
                chain: Chain::Base,
            },
            ContractEntry {
                address: address!("0x1111111254eeb25477b68fb85ed929f73a960582"),
                provider: "1inch v5",
                chain: Chain::SmartChain,
            },
            ContractEntry {
                address: address!("0xc82384da1318f167ff453760eb71dd6012896240"),
                provider: "0x Protocol",
                chain: Chain::Optimism,
            },
            ContractEntry {
                address: address!("0x19ceead7105607cd444f5ad10dd51356436095a1"),
                provider: "Odos v2",
                chain: Chain::Base,
            },
            ContractEntry {
                address: address!("0xa3d370e8a4180828f6756cb8dce359cf21d9d6f7"),
                provider: "0x Protocol",
                chain: Chain::Polygon,
            },
            ContractEntry {
                address: address!("0x246475e1f63d8e26d6f4fb6029033da8831ed396"),
                provider: "0x Protocol",
                chain: Chain::Arbitrum,
            },
            ContractEntry {
                address: address!("0x1111111254eeb25477b68fb85ed929f73a960582"),
                provider: "1inch v5",
                chain: Chain::Ethereum,
            },
            ContractEntry {
                address: address!("0x779a74436eda060911b2c4f209d34ea155f3df09"),
                provider: "0x Protocol",
                chain: Chain::SmartChain,
            },
            ContractEntry {
                address: address!("0x1111111254eeb25477b68fb85ed929f73a960582"),
                provider: "1inch v5",
                chain: Chain::Base,
            },
            ContractEntry {
                address: address!("0x111111125421ca6dc452d289314280a0f8842a65"),
                provider: "1inch v6",
                chain: Chain::Arbitrum,
            },
            ContractEntry {
                address: address!("0x5c9bdc801a600c006c388fc032dcb27355154cc9"),
                provider: "0x Protocol",
                chain: Chain::Base,
            },
            ContractEntry {
                address: address!("0x5418226af9c8d5d287a78fbbbcd337b86ec07d61"),
                provider: "0x Protocol",
                chain: Chain::Ethereum,
            },
            ContractEntry {
                address: address!("0x111111125421ca6dc452d289314280a0f8842a65"),
                provider: "1inch v6",
                chain: Chain::Ethereum,
            },
            ContractEntry {
                address: address!("0x111111125421ca6dc452d289314280a0f8842a65"),
                provider: "1inch v6",
                chain: Chain::Polygon,
            },
            ContractEntry {
                address: address!("0x402867b638339ad8bec6e5373cfa95da0b462c85"),
                provider: "0x Protocol",
                chain: Chain::Optimism,
            },
            ContractEntry {
                address: address!("0x5435453c2e5d31908fa1667f583e37ae26c9f382"),
                provider: "0x Protocol",
                chain: Chain::Unichain,
            },
            ContractEntry {
                address: address!("0x1231deb6f5749ef6ce6943a275a1d3e7486f4eae"),
                provider: "LI.FI v2",
                chain: Chain::Optimism,
            },
            ContractEntry {
                address: address!("0xd8014f15a920bf9edfdb87159ee10cadc07fcb53"),
                provider: "0x Protocol",
                chain: Chain::Optimism,
            },
            ContractEntry {
                address: address!("0x6131b5fae19ea4f9d964eac0408e4408b66337b5"),
                provider: "KyberSwap Meta v2",
                chain: Chain::SmartChain,
            },
            ContractEntry {
                address: address!("0x6a000f20005980200259b80c5102003040001068"),
                provider: "ParaSwap v6",
                chain: Chain::AvalancheC,
            },
            ContractEntry {
                address: address!("0x7a250d5630b4cf539739df2c5dacb4c659f2488d"),
                provider: "Uniswap v2",
                chain: Chain::Ethereum,
            },
        ];

        Self { entries }
    }

    pub fn get_by_address(&self, address: &Address) -> Option<&ContractEntry> {
        self.entries.iter().find(|entry| entry.address == *address)
    }

    pub fn get_by_chain(&self, chain: Chain) -> Vec<&ContractEntry> {
        self.entries.iter().filter(|entry| entry.chain == chain).collect()
    }
}

impl Default for ContractRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_by_address() {
        let registry = ContractRegistry::new();
        let test_address = address!("0x5968feacba91d55010975e0cfe8acfc32664ad33");

        let entry = registry.get_by_address(&test_address).unwrap();

        assert_eq!(entry.provider, "PancakeSwap v3");
        assert_eq!(entry.chain, Chain::SmartChain);
    }

    #[test]
    fn test_get_by_chain() {
        let registry = ContractRegistry::new();
        let bnb_contracts = registry.get_by_chain(Chain::SmartChain);

        for contract in bnb_contracts {
            assert_eq!(contract.chain, Chain::SmartChain);
        }
    }
}
