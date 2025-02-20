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
        | Chain::Hyperliquid => Some(EthereumAddress::parse(&token_id)?.to_checksum()),
        Chain::Solana | Chain::Sui | Chain::Ton => Some(token_id),
        Chain::Tron => (token_id.len() == 34 && token_id.starts_with('T')).then_some(token_id),
        Chain::Xrp => token_id.starts_with('r').then_some(token_id),
        Chain::Algorand => token_id.parse::<i32>().ok().map(|token_id| token_id.to_string()),
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
        | Chain::Stellar
        | Chain::Polkadot
        | Chain::Cardano => None,
    }
}
