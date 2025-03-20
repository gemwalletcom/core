use crate::swapper::SwapperError;
use primitives::Chain;

pub fn get_debridge_chain_id(chain: &Chain) -> Result<String, SwapperError> {
    let chain_id = match chain {
        Chain::Ethereum
        | Chain::Optimism
        | Chain::SmartChain
        | Chain::Polygon
        | Chain::Fantom
        | Chain::Base
        | Chain::Arbitrum
        | Chain::AvalancheC
        | Chain::Linea => chain.network_id(),
        Chain::Solana => "7565164",
        Chain::Gnosis => "100000002",
        Chain::Sonic => "100000014",
        Chain::Abstract => "100000017",
        Chain::Berachain => "100000020",
        Chain::Hyperliquid => "100000022",
        _ => return Err(SwapperError::NotSupportedChain),
    };

    Ok(chain_id.to_string())
}
