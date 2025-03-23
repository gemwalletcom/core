use std::collections::HashMap;

use primitives::Chain;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum WormholeId {
    Solana = 1,
    Ethereum = 2,
    SmartChain = 4,
    Polygon = 5,
    Sui = 21,
    Arbitrum = 23,
    Optimism = 24,
    Base = 30,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MayanSwiftDeployment {
    pub address: String,
    pub wormhole_id: WormholeId,
}

// https://docs.mayan.finance/architecture/swift#swift-contract-addresses
pub fn get_swift_providers() -> HashMap<Chain, MayanSwiftDeployment> {
    HashMap::from([
        (
            Chain::Solana,
            MayanSwiftDeployment {
                address: "BLZRi6frs4X4DNLw56V4EXai1b6QVESN1BhHBTYM9VcY".to_string(),
                wormhole_id: WormholeId::Solana,
            },
        ),
        (
            Chain::Ethereum,
            MayanSwiftDeployment {
                address: "0xC38e4e6A15593f908255214653d3D947CA1c2338".to_string(),
                wormhole_id: WormholeId::Ethereum,
            },
        ),
        (
            Chain::SmartChain,
            MayanSwiftDeployment {
                address: "0xC38e4e6A15593f908255214653d3D947CA1c2338".to_string(),
                wormhole_id: WormholeId::SmartChain,
            },
        ),
        (
            Chain::Polygon,
            MayanSwiftDeployment {
                address: "0xC38e4e6A15593f908255214653d3D947CA1c2338".to_string(),
                wormhole_id: WormholeId::Polygon,
            },
        ),
        (
            Chain::Arbitrum,
            MayanSwiftDeployment {
                address: "0xC38e4e6A15593f908255214653d3D947CA1c2338".to_string(),
                wormhole_id: WormholeId::Arbitrum,
            },
        ),
        (
            Chain::Optimism,
            MayanSwiftDeployment {
                address: "0xC38e4e6A15593f908255214653d3D947CA1c2338".to_string(),
                wormhole_id: WormholeId::Optimism,
            },
        ),
        (
            Chain::Base,
            MayanSwiftDeployment {
                address: "0xC38e4e6A15593f908255214653d3D947CA1c2338".to_string(),
                wormhole_id: WormholeId::Base,
            },
        ),
        (
            Chain::Sui,
            MayanSwiftDeployment {
                address: "0xC38e4e6A15593f908255214653d3D947CA1c2338".to_string(),
                wormhole_id: WormholeId::Sui,
            },
        ),
    ])
}

pub fn get_swift_deployment_chains() -> Vec<Chain> {
    get_swift_providers().keys().cloned().collect()
}

pub fn get_swift_deployment_by_chain(chain: Chain) -> Option<MayanSwiftDeployment> {
    get_swift_providers().get(&chain).cloned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_swift_provider_address() {
        // Test all supported chains
        assert_eq!(
            get_swift_deployment_by_chain(Chain::Solana),
            Some(MayanSwiftDeployment {
                address: "BLZRi6frs4X4DNLw56V4EXai1b6QVESN1BhHBTYM9VcY".to_string(),
                wormhole_id: WormholeId::Solana,
            })
        );

        let evm_address = "0xC38e4e6A15593f908255214653d3D947CA1c2338";

        assert_eq!(get_swift_deployment_by_chain(Chain::Ethereum).map(|x| x.address), Some(evm_address.to_string()));
        assert_eq!(
            get_swift_deployment_by_chain(Chain::SmartChain).map(|x| x.address),
            Some(evm_address.to_string())
        );
        assert_eq!(get_swift_deployment_by_chain(Chain::Polygon).map(|x| x.address), Some(evm_address.to_string()));
        assert_eq!(get_swift_deployment_by_chain(Chain::Arbitrum).map(|x| x.address), Some(evm_address.to_string()));
        assert_eq!(get_swift_deployment_by_chain(Chain::Optimism).map(|x| x.address), Some(evm_address.to_string()));

        // Test unsupported chain
        assert_eq!(get_swift_deployment_by_chain(Chain::Sui), None);
    }

    #[test]
    fn test_chain_ids() {
        // Verify chain IDs match the provided table, and that they are in order
        assert_eq!(get_swift_deployment_by_chain(Chain::Solana).map(|x| x.wormhole_id), Some(WormholeId::Solana));
        assert_eq!(
            get_swift_deployment_by_chain(Chain::Ethereum).map(|x| x.wormhole_id),
            Some(WormholeId::Ethereum)
        );
        assert_eq!(
            get_swift_deployment_by_chain(Chain::SmartChain).map(|x| x.wormhole_id),
            Some(WormholeId::SmartChain)
        );
        assert_eq!(get_swift_deployment_by_chain(Chain::Polygon).map(|x| x.wormhole_id), Some(WormholeId::Polygon));
        // assert_eq!(get_swift_deployment_by_chain(Chain::Avalanche).map(|x| x.wormhole_id), Some(6));
        assert_eq!(
            get_swift_deployment_by_chain(Chain::Arbitrum).map(|x| x.wormhole_id),
            Some(WormholeId::Arbitrum)
        );
        assert_eq!(
            get_swift_deployment_by_chain(Chain::Optimism).map(|x| x.wormhole_id),
            Some(WormholeId::Optimism)
        );
    }
}
