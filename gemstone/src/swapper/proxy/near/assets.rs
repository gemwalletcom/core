use primitives::Chain;
use std::collections::HashMap;

pub fn parse_near_asset_chain(asset_id: &str) -> Option<Chain> {
    NEAR_INTENTS_TO_CHAIN.get(asset_id).copied()
}

static NEAR_INTENTS_TO_CHAIN: std::sync::LazyLock<HashMap<&'static str, Chain>> = std::sync::LazyLock::new(|| {
    let mut mapping = HashMap::new();

    mapping.insert("nep141:wrap.near", Chain::Near);
    mapping.insert("nep141:eth.omft.near", Chain::Ethereum);
    mapping.insert("nep141:eth-0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48.omft.near", Chain::Ethereum);
    mapping.insert("nep141:eth-0xdac17f958d2ee523a2206206994597c13d831ec7.omft.near", Chain::Ethereum);
    mapping.insert("nep141:btc.omft.near", Chain::Bitcoin);
    mapping.insert("nep141:sol.omft.near", Chain::Solana);
    mapping.insert("nep141:sol-5ce3bf3a31af18be40ba30f721101b4341690186.omft.near", Chain::Solana);
    mapping.insert("nep141:sol-c800a4bd850783ccb82c2b2c7e84175443606352.omft.near", Chain::Solana);
    mapping.insert("nep141:sui.omft.near", Chain::Sui);
    mapping.insert("nep141:arb.omft.near", Chain::Arbitrum);
    mapping.insert("nep141:arb-0xaf88d065e77c8cc2239327c5edb3a432268e5831.omft.near", Chain::Arbitrum);
    mapping.insert("nep141:arb-0xfd086bc7cd5c481dcc9c85ebe478a1c0b69fcbb9.omft.near", Chain::Arbitrum);
    mapping.insert("nep141:base.omft.near", Chain::Base);
    mapping.insert("nep141:base-0x833589fcd6edb6e08f4c7c32d4f71b54bda02913.omft.near", Chain::Base);
    mapping.insert("nep245:v2_1.omni.hot.tg:10_11111111111111111111", Chain::Optimism);
    mapping.insert("nep245:v2_1.omni.hot.tg:10_A2ewyUyDp6qsue1jqZsGypkCxRJ", Chain::Optimism);
    mapping.insert("nep245:v2_1.omni.hot.tg:10_359RPSJVdTxwTJT9TyGssr2rFoWo", Chain::Optimism);
    mapping.insert("nep245:v2_1.omni.hot.tg:43114_11111111111111111111", Chain::AvalancheC);
    mapping.insert("nep245:v2_1.omni.hot.tg:43114_3atVJH3r5c4GqiSYmg9fECvjc47o", Chain::AvalancheC);
    mapping.insert("nep245:v2_1.omni.hot.tg:43114_372BeH7ENZieCaabwkbWkBiTTgXp", Chain::AvalancheC);
    mapping.insert("nep245:v2_1.omni.hot.tg:56_11111111111111111111", Chain::SmartChain);
    mapping.insert("nep245:v2_1.omni.hot.tg:56_2w93GqMcEmQFDru84j3HZZWt557r", Chain::SmartChain);
    mapping.insert("nep245:v2_1.omni.hot.tg:56_2CMMyVTGZkeyNZTSvS5sarzfir6g", Chain::SmartChain);
    mapping.insert("nep245:v2_1.omni.hot.tg:137_11111111111111111111", Chain::Polygon);
    mapping.insert("nep245:v2_1.omni.hot.tg:137_qiStmoQJDQPTebaPjgx5VBxZv6L", Chain::Polygon);
    mapping.insert("nep245:v2_1.omni.hot.tg:137_3hpYoaLtt8MP1Z2GH1U473DMRKgr", Chain::Polygon);
    mapping.insert("nep245:v2_1.omni.hot.tg:1117_", Chain::Ton);
    mapping.insert("nep245:v2_1.omni.hot.tg:1117_3tsdfyziyc7EJbP2aULWSKU4toBaAcN4FdTgfm5W1mC4ouR", Chain::Ton);
    mapping.insert("nep141:tron.omft.near", Chain::Tron);
    mapping.insert("nep141:tron-d28a265909efecdcee7c5028585214ea0b96f015.omft.near", Chain::Tron);
    mapping.insert("nep141:doge.omft.near", Chain::Doge);
    mapping.insert("nep141:xrp.omft.near", Chain::Xrp);
    mapping.insert("nep141:cardano.omft.near", Chain::Cardano);
    mapping.insert("nep141:bera.omft.near", Chain::Berachain);
    mapping.insert("nep141:gnosis.omft.near", Chain::Gnosis);
    mapping.insert("nep141:gnosis-0x2a22f9c3b484c3629090feed35f17ff8f88f76f0.omft.near", Chain::Gnosis);

    mapping
});

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_bitcoin_asset() {
        assert_eq!(parse_near_asset_chain("nep141:btc.omft.near"), Some(Chain::Bitcoin));
    }

    #[test]
    fn test_parse_ethereum_asset() {
        assert_eq!(parse_near_asset_chain("nep141:eth.omft.near"), Some(Chain::Ethereum));
    }

    #[test]
    fn test_parse_ethereum_usdc() {
        assert_eq!(
            parse_near_asset_chain("nep141:eth-0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48.omft.near"),
            Some(Chain::Ethereum)
        );
    }

    #[test]
    fn test_parse_solana_asset() {
        assert_eq!(parse_near_asset_chain("nep141:sol.omft.near"), Some(Chain::Solana));
    }

    #[test]
    fn test_parse_optimism_asset() {
        assert_eq!(parse_near_asset_chain("nep245:v2_1.omni.hot.tg:10_11111111111111111111"), Some(Chain::Optimism));
    }

    #[test]
    fn test_parse_unknown_asset() {
        assert_eq!(parse_near_asset_chain("unknown:asset.id"), None);
    }
}
