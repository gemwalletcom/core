use alloy_primitives::Address;
use primitives::Chain;
use std::str::FromStr;

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
        | Chain::HyperCore
        | Chain::Monad => Address::from_str(&token_id).ok().map(|address| address.to_checksum(None)),
        Chain::Solana | Chain::Ton | Chain::Near => Some(token_id),
        Chain::Tron => (token_id.len() == 34 && token_id.starts_with('T')).then_some(token_id),
        Chain::Xrp => {
            if let Some((_, addr)) = token_id.split_once('.') {
                if addr.starts_with('r') {
                    return Some(addr.to_string());
                }
            }
            token_id.starts_with('r').then_some(token_id)
        }
        Chain::Algorand => token_id.parse::<i32>().ok().map(|token_id| token_id.to_string()),
        Chain::Sui => {
            if token_id.len() >= 64
                && token_id.starts_with("0x")
                && token_id.matches("::").count() == 2
                && !token_id.starts_with("0x0000000000000000000000000000000000000000000000000000000000000002")
            {
                Some(token_id)
            } else {
                None
            }
        }
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

        assert_eq!(formatted_valid_token_id.unwrap(), "0x1234567890AbcdEF1234567890aBcdef12345678");
        assert_eq!(format_token_id(chain, "0x123".to_string()), None);
    }

    #[test]
    fn test_format_token_id_sui() {
        let chain = Chain::Sui;
        assert_eq!(
            format_token_id(chain, "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string()),
            None
        );
        assert_eq!(format_token_id(chain, "0x2::sui::SUI".to_string()), None);
        assert_eq!(
            format_token_id(
                chain,
                "0x356a26eb9e012a68958082340d4c4116e7f55615cf27affcff209cf0ae544f59::wal::WAL".to_string()
            ),
            Some("0x356a26eb9e012a68958082340d4c4116e7f55615cf27affcff209cf0ae544f59::wal::WAL".to_string())
        );
        assert_eq!(
            format_token_id(
                chain,
                "0x0000000000000000000000000000000000000000000000000000000000000002::sui::SUI".to_string()
            ),
            None
        );
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

    #[test]
    fn test_format_token_id_xrp() {
        let chain = Chain::Xrp;

        assert_eq!(
            format_token_id(chain, "534F4C4F00000000000000000000000000000000.rsoLo2S1kiGeCcn6hCUXVrCpGMWLrRrLZz".to_string()),
            Some("rsoLo2S1kiGeCcn6hCUXVrCpGMWLrRrLZz".to_string())
        );
        assert_eq!(
            format_token_id(chain, "rsoLo2S1kiGeCcn6hCUXVrCpGMWLrRrLZz".to_string()),
            Some("rsoLo2S1kiGeCcn6hCUXVrCpGMWLrRrLZz".to_string())
        );
    }
}
