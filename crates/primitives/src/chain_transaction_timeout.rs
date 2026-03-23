use crate::{Chain, ChainType};

pub fn chain_transaction_timeout_seconds(chain: Chain) -> u32 {
    match chain.chain_type() {
        ChainType::Bitcoin => 1_209_600,
        ChainType::Solana => chain.block_time() * 150,
        ChainType::Ethereum => chain.block_time() * 120,
        ChainType::Cosmos
        | ChainType::Ton
        | ChainType::Tron
        | ChainType::Aptos
        | ChainType::Sui
        | ChainType::Xrp
        | ChainType::Near
        | ChainType::Stellar
        | ChainType::Algorand
        | ChainType::Polkadot
        | ChainType::Cardano
        | ChainType::HyperCore => chain.block_time() * 600,
    }
}

#[cfg(test)]
mod tests {
    use super::chain_transaction_timeout_seconds;
    use crate::Chain;

    #[test]
    fn test_chain_transaction_timeout_seconds() {
        assert_eq!(chain_transaction_timeout_seconds(Chain::Bitcoin), 1_209_600);
        assert_eq!(chain_transaction_timeout_seconds(Chain::Solana), Chain::Solana.block_time() * 150);
        assert_eq!(chain_transaction_timeout_seconds(Chain::Ethereum), Chain::Ethereum.block_time() * 120);
        assert_eq!(chain_transaction_timeout_seconds(Chain::Cosmos), Chain::Cosmos.block_time() * 600);
    }
}
