use gem_evm::address::EthereumAddress;
use primitives::Chain;

pub fn format_token_id(chain: Chain, token_id: String) -> Option<String> {
    match chain {
        Chain::Ethereum
        | Chain::SmartChain
        | Chain::Polygon
        | Chain::Arbitrum
        | Chain::Optimism
        | Chain::Base
        | Chain::AvalancheC
        | Chain::OpBNB
        | Chain::Fantom
        | Chain::Gnosis
        | Chain::Manta
        | Chain::Blast
        | Chain::ZkSync
        | Chain::Linea
        | Chain::Mantle
        | Chain::Celo
        | Chain::World
        | Chain::Sonic
        | Chain::Abstract
        | Chain::Berachain
        | Chain::Ink
        | Chain::Unichain
        | Chain::Hyperliquid
        | Chain::Monad => Some(EthereumAddress::parse(&token_id)?.to_checksum()),
        Chain::Solana | Chain::Ton => Some(token_id),
        Chain::Tron => (token_id.len() == 34 && token_id.starts_with('T')).then_some(token_id),
        Chain::Xrp => token_id.starts_with('r').then_some(token_id),
        Chain::Algorand => token_id.parse::<i32>().ok().map(|token_id| token_id.to_string()),
        Chain::Sui => (token_id.len() >= 64 && token_id.starts_with("0x")).then_some(token_id),
        Chain::Stellar => (token_id.len() == 56 && token_id.starts_with('G')).then_some(token_id),
        Chain::Bitcoin
        | Chain::BitcoinCash
        | Chain::Litecoin
        | Chain::Thorchain
        | Chain::Cosmos
        | Chain::Osmosis
        | Chain::Celestia
        | Chain::Doge
        | Chain::Aptos
        | Chain::Injective
        | Chain::Noble
        | Chain::Sei
        | Chain::Near
        | Chain::Polkadot
        | Chain::Cardano => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_token_id_ethereum() {
        let chain = Chain::Ethereum;

        let valid_token_id = "0x1234567890abcdef1234567890abcdef12345678".to_string();
        let formatted_valid_token_id = format_token_id(chain, valid_token_id.clone());
        assert_eq!(formatted_valid_token_id, Some(EthereumAddress::parse(&valid_token_id).unwrap().to_checksum()));

        assert_eq!(format_token_id(chain, "0x123".to_string()), None);
    }

    #[test]
    fn test_format_token_id_sui() {
        let chain = Chain::Sui;

        let valid_token_id = "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string();
        let formatted_valid_token_id = format_token_id(chain, "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string().clone());
        assert_eq!(formatted_valid_token_id, Some(valid_token_id));

        assert_eq!(format_token_id(chain, "0x2::sui::SUI".to_string()), None);
    }

    #[test]
    fn test_format_token_id_tron() {
        let chain = Chain::Tron;

        let valid_token_id = "TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t".to_string();
        let formatted_valid_token_id = format_token_id(chain, valid_token_id.clone());
        assert_eq!(formatted_valid_token_id, Some(valid_token_id));

        assert_eq!(format_token_id(chain, "1234567890123456789012345678901234".to_string()), None);
        assert_eq!(format_token_id(chain, "T123".to_string()), None);
    }
}
