use primitives::Chain;

pub const BASE_URL: &str = "https://web3.okx.com";

pub const EVM_NATIVE_TOKEN_ADDRESS: &str = "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE";
pub const SOLANA_NATIVE_TOKEN_ADDRESS: &str = "11111111111111111111111111111111";

const DEFAULT_EVM_GAS_LIMIT: u64 = 920_000;

const SOLANA_DEX_IDS: &str = "277,278,279,343,72,103,284,338,372,403,444,483,357,345";

pub fn chain_index(chain: Chain) -> Option<&'static str> {
    match chain {
        Chain::Solana => Some("501"),
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
        | Chain::Sonic
        | Chain::Abstract
        | Chain::Berachain
        | Chain::Unichain
        | Chain::Monad
        | Chain::XLayer => Some(chain.config().network_id),
        _ => None,
    }
}

pub fn dex_ids(chain: Chain) -> Option<&'static str> {
    if chain == Chain::Solana { Some(SOLANA_DEX_IDS) } else { None }
}

pub fn evm_gas_limit(chain: Chain) -> u64 {
    match chain {
        Chain::Manta => 600_000,
        Chain::ZkSync => 2_000_000,
        Chain::Mantle => 2_000_000_000,
        _ => DEFAULT_EVM_GAS_LIMIT,
    }
}
