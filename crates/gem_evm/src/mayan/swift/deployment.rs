use std::collections::HashMap;

use primitives::Chain;

#[derive(Debug, Clone, PartialEq)]
pub struct MayanSwiftDeployment {
    pub address: String,
    pub wormhole_id: u64,
}

pub fn get_swift_providers() -> HashMap<Chain, MayanSwiftDeployment> {
    let mut map = HashMap::new();
    map.insert(
        Chain::Solana,
        MayanSwiftDeployment {
            address: "BLZRi6frs4X4DNLw56V4EXai1b6QVESN1BhHBTYM9VcY".to_string(),
            wormhole_id: 1,
        },
    );
    map.insert(
        Chain::Ethereum,
        MayanSwiftDeployment {
            address: "0xC38e4e6A15593f908255214653d3D947CA1c2338".to_string(),
            wormhole_id: 2,
        },
    );
    map.insert(
        Chain::SmartChain,
        MayanSwiftDeployment {
            address: "0xC38e4e6A15593f908255214653d3D947CA1c2338".to_string(),
            wormhole_id: 4,
        },
    );
    map.insert(
        Chain::Polygon,
        MayanSwiftDeployment {
            address: "0xC38e4e6A15593f908255214653d3D947CA1c2338".to_string(),
            wormhole_id: 5,
        },
    );
    map.insert(
        Chain::Arbitrum,
        MayanSwiftDeployment {
            address: "0xC38e4e6A15593f908255214653d3D947CA1c2338".to_string(),
            wormhole_id: 23,
        },
    );
    map.insert(
        Chain::Optimism,
        MayanSwiftDeployment {
            address: "0xC38e4e6A15593f908255214653d3D947CA1c2338".to_string(),
            wormhole_id: 24,
        },
    );
    map.insert(
        Chain::Base,
        MayanSwiftDeployment {
            address: "0xC38e4e6A15593f908255214653d3D947CA1c2338".to_string(),
            wormhole_id: 30,
        },
    );

    map
}

pub fn get_swift_deployment_chains() -> Vec<Chain> {
    get_swift_providers().keys().cloned().collect()
}

pub fn get_swift_deployment_by_chain(chain: Chain) -> Option<MayanSwiftDeployment> {
    get_swift_providers().get(&chain).map(|x| x.clone())
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
                wormhole_id: 1,
            })
        );

        let evm_address = "0xC38e4e6A15593f908255214653d3D947CA1c2338";

        assert_eq!(get_swift_deployment_by_chain(Chain::Ethereum).map(|x| x.address), Some(evm_address.to_string()));
        assert_eq!(
            get_swift_deployment_by_chain(Chain::SmartChain).map(|x| x.address),
            Some(evm_address.to_string())
        );
        assert_eq!(get_swift_deployment_by_chain(Chain::Polygon).map(|x| x.address), Some(evm_address.to_string()));
        // assert_eq!(get_swift_deployment_by_chain(Chain::Avalanche).map(|x| x.address), Some(evm_address.to_string()));
        assert_eq!(get_swift_deployment_by_chain(Chain::Arbitrum).map(|x| x.address), Some(evm_address.to_string()));
        assert_eq!(get_swift_deployment_by_chain(Chain::Optimism).map(|x| x.address), Some(evm_address.to_string()));

        // Test unsupported chain
        assert_eq!(get_swift_deployment_by_chain(Chain::Sui), None);
    }

    #[test]
    fn test_chain_ids() {
        // Verify chain IDs match the provided table, and that they are in order
        assert_eq!(get_swift_deployment_by_chain(Chain::Solana).map(|x| x.wormhole_id), Some(1));
        assert_eq!(get_swift_deployment_by_chain(Chain::Ethereum).map(|x| x.wormhole_id), Some(2));
        assert_eq!(get_swift_deployment_by_chain(Chain::SmartChain).map(|x| x.wormhole_id), Some(4));
        assert_eq!(get_swift_deployment_by_chain(Chain::Polygon).map(|x| x.wormhole_id), Some(5));
        // assert_eq!(get_swift_deployment_by_chain(Chain::Avalanche).map(|x| x.wormhole_id), Some(6));
        assert_eq!(get_swift_deployment_by_chain(Chain::Arbitrum).map(|x| x.wormhole_id), Some(23));
        assert_eq!(get_swift_deployment_by_chain(Chain::Optimism).map(|x| x.wormhole_id), Some(24));
    }
}
