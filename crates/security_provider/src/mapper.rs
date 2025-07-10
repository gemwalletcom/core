use primitives::Chain;

pub fn chain_to_provider_id(chain: Chain) -> String {
    match chain {
        Chain::Ethereum => "1".to_string(),
        Chain::SmartChain => "56".to_string(),
        Chain::Polygon => "137".to_string(),
        Chain::Arbitrum => "42161".to_string(),
        Chain::Optimism => "10".to_string(),
        Chain::Base => "8453".to_string(),
        Chain::AvalancheC => "43114".to_string(),
        Chain::OpBNB => "204".to_string(),
        Chain::Fantom => "250".to_string(),
        Chain::Gnosis => "100".to_string(),
        Chain::Blast => "81457".to_string(),
        Chain::ZkSync => "324".to_string(),
        Chain::Linea => "59144".to_string(),
        Chain::Mantle => "5000".to_string(),
        Chain::Celo => "42220".to_string(),
        Chain::Manta => "169".to_string(),
        Chain::World => "480".to_string(),
        // Default to Ethereum for unsupported chains
        _ => "1".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chain_to_provider_id() {
        assert_eq!(chain_to_provider_id(Chain::Ethereum), "1");
        assert_eq!(chain_to_provider_id(Chain::SmartChain), "56");
        assert_eq!(chain_to_provider_id(Chain::Polygon), "137");
        assert_eq!(chain_to_provider_id(Chain::Arbitrum), "42161");
        assert_eq!(chain_to_provider_id(Chain::Optimism), "10");
        assert_eq!(chain_to_provider_id(Chain::Base), "8453");
        assert_eq!(chain_to_provider_id(Chain::Bitcoin), "1"); // Unsupported, defaults to Ethereum
    }
}
