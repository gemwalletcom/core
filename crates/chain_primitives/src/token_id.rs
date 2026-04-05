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
        | Chain::SeiEvm
        | Chain::Abstract
        | Chain::Berachain
        | Chain::Ink
        | Chain::Unichain
        | Chain::Hyperliquid
        | Chain::HyperCore
        | Chain::Plasma
        | Chain::Monad
        | Chain::XLayer
        | Chain::Stable => Address::from_str(&token_id).ok().map(|address| address.to_checksum(None)),
        Chain::Solana | Chain::Ton | Chain::Near => Some(token_id),
        Chain::Tron => (token_id.len() == 34 && token_id.starts_with('T')).then_some(token_id),
        Chain::Xrp => {
            if let Some((_, addr)) = token_id.split_once('.')
                && addr.starts_with('r')
            {
                return Some(addr.to_string());
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
        Chain::Stellar => {
            if let Some((issuer, symbol)) = token_id.split_once("::") {
                (issuer.len() == 56 && issuer.starts_with('G') && !symbol.is_empty()).then_some(token_id)
            } else {
                None
            }
        }
        Chain::Bitcoin
        | Chain::BitcoinCash
        | Chain::Litecoin
        | Chain::Thorchain
        | Chain::Cosmos
        | Chain::Osmosis
        | Chain::Celestia
        | Chain::Doge
        | Chain::Zcash
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
    use primitives::asset_constants::{STELLAR_USDC_TOKEN_ID, SUI_WAL_TOKEN_ID, TRON_USDT_TOKEN_ID};

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
        assert_eq!(format_token_id(chain, SUI_WAL_TOKEN_ID.to_string()), Some(SUI_WAL_TOKEN_ID.to_string()));
        assert_eq!(
            format_token_id(chain, "0x0000000000000000000000000000000000000000000000000000000000000002::sui::SUI".to_string()),
            None
        );
    }

    #[test]
    fn test_format_token_id_tron() {
        let chain = Chain::Tron;

        let valid_token_id = TRON_USDT_TOKEN_ID.to_string();
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

    #[test]
    fn test_format_token_id_stellar() {
        let chain = Chain::Stellar;

        assert_eq!(format_token_id(chain, STELLAR_USDC_TOKEN_ID.to_string()), Some(STELLAR_USDC_TOKEN_ID.to_string()));
        assert_eq!(format_token_id(chain, "GA5ZSEJYB37JRC5AVCIA5MOP4RHTM335X2KGX3IHOJAPP5RE34K4KZVN".to_string()), None);
        assert_eq!(format_token_id(chain, "invalid".to_string()), None);
        assert_eq!(format_token_id(chain, "GA5ZSEJYB37JRC5AVCIA5MOP4RHTM335X2KGX3IHOJAPP5RE34K4KZVN::".to_string()), None);
    }
}
