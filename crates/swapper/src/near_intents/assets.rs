use crate::{SwapperError, SwapperQuoteAsset, models::SwapperChainAsset};
use primitives::{
    AssetId, Chain,
    asset_constants::{
        USDC_ARB_ASSET_ID, USDC_AVAX_ASSET_ID, USDC_BASE_ASSET_ID, USDC_ETH_ASSET_ID, USDC_GNOSIS_ASSET_ID, USDC_OP_ASSET_ID, USDC_POLYGON_ASSET_ID,
        USDC_SOLANA_ASSET_ID, USDT_ARB_ASSET_ID, USDT_AVAX_ASSET_ID, USDT_ETH_ASSET_ID, USDT_OP_ASSET_ID, USDT_POLYGON_ASSET_ID, USDT_SMARTCHAIN_ASSET_ID,
        USDT_SOLANA_ASSET_ID, USDT_TON_ASSET_ID, USDT_TRON_ASSET_ID,
    },
};
use std::{collections::HashMap, sync::LazyLock};

pub const NEAR_INTENTS_WRAP_NEAR: &str = "nep141:wrap.near";
pub const NEAR_INTENTS_ETH_NATIVE: &str = "nep141:eth.omft.near";
pub const NEAR_INTENTS_ETH_USDC: &str = "nep141:eth-0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48.omft.near";
pub const NEAR_INTENTS_ETH_USDT: &str = "nep141:eth-0xdac17f958d2ee523a2206206994597c13d831ec7.omft.near";
pub const NEAR_INTENTS_BTC_NATIVE: &str = "nep141:btc.omft.near";
pub const NEAR_INTENTS_SOL_NATIVE: &str = "nep141:sol.omft.near";
pub const NEAR_INTENTS_SOL_USDC: &str = "nep141:sol-5ce3bf3a31af18be40ba30f721101b4341690186.omft.near";
pub const NEAR_INTENTS_SOL_USDT: &str = "nep141:sol-c800a4bd850783ccb82c2b2c7e84175443606352.omft.near";
pub const NEAR_INTENTS_SUI_NATIVE: &str = "nep141:sui.omft.near";
pub const NEAR_INTENTS_ARB_NATIVE: &str = "nep141:arb.omft.near";
pub const NEAR_INTENTS_ARB_USDC: &str = "nep141:arb-0xaf88d065e77c8cc2239327c5edb3a432268e5831.omft.near";
pub const NEAR_INTENTS_ARB_USDT: &str = "nep141:arb-0xfd086bc7cd5c481dcc9c85ebe478a1c0b69fcbb9.omft.near";
pub const NEAR_INTENTS_BASE_NATIVE: &str = "nep141:base.omft.near";
pub const NEAR_INTENTS_BASE_USDC: &str = "nep141:base-0x833589fcd6edb6e08f4c7c32d4f71b54bda02913.omft.near";
pub const NEAR_INTENTS_OPT_NATIVE: &str = "nep245:v2_1.omni.hot.tg:10_11111111111111111111";
pub const NEAR_INTENTS_OPT_USDC: &str = "nep245:v2_1.omni.hot.tg:10_A2ewyUyDp6qsue1jqZsGypkCxRJ";
pub const NEAR_INTENTS_OPT_USDT: &str = "nep245:v2_1.omni.hot.tg:10_359RPSJVdTxwTJT9TyGssr2rFoWo";
pub const NEAR_INTENTS_AVAX_NATIVE: &str = "nep245:v2_1.omni.hot.tg:43114_11111111111111111111";
pub const NEAR_INTENTS_AVAX_USDC: &str = "nep245:v2_1.omni.hot.tg:43114_3atVJH3r5c4GqiSYmg9fECvjc47o";
pub const NEAR_INTENTS_AVAX_USDT: &str = "nep245:v2_1.omni.hot.tg:43114_372BeH7ENZieCaabwkbWkBiTTgXp";
pub const NEAR_INTENTS_BSC_NATIVE: &str = "nep245:v2_1.omni.hot.tg:56_11111111111111111111";
#[allow(unused)]
pub const NEAR_INTENTS_BSC_USDC: &str = "nep245:v2_1.omni.hot.tg:56_2w93GqMcEmQFDru84j3HZZWt557r";
pub const NEAR_INTENTS_BSC_USDT: &str = "nep245:v2_1.omni.hot.tg:56_2CMMyVTGZkeyNZTSvS5sarzfir6g";
pub const NEAR_INTENTS_POL_NATIVE: &str = "nep245:v2_1.omni.hot.tg:137_11111111111111111111";
pub const NEAR_INTENTS_POL_USDC: &str = "nep245:v2_1.omni.hot.tg:137_qiStmoQJDQPTebaPjgx5VBxZv6L";
pub const NEAR_INTENTS_POL_USDT: &str = "nep245:v2_1.omni.hot.tg:137_3hpYoaLtt8MP1Z2GH1U473DMRKgr";
pub const NEAR_INTENTS_TON_NATIVE: &str = "nep245:v2_1.omni.hot.tg:1117_";
pub const NEAR_INTENTS_TON_USDT: &str = "nep245:v2_1.omni.hot.tg:1117_3tsdfyziyc7EJbP2aULWSKU4toBaAcN4FdTgfm5W1mC4ouR";
pub const NEAR_INTENTS_TRON_NATIVE: &str = "nep141:tron.omft.near";
pub const NEAR_INTENTS_TRON_USDT: &str = "nep141:tron-d28a265909efecdcee7c5028585214ea0b96f015.omft.near";
pub const NEAR_INTENTS_DOGE_NATIVE: &str = "nep141:doge.omft.near";
pub const NEAR_INTENTS_XRP_NATIVE: &str = "nep141:xrp.omft.near";
pub const NEAR_INTENTS_CARDANO_NATIVE: &str = "nep141:cardano.omft.near";
pub const NEAR_INTENTS_BERA_NATIVE: &str = "nep141:bera.omft.near";
pub const NEAR_INTENTS_GNOSIS_NATIVE: &str = "nep141:gnosis.omft.near";
pub const NEAR_INTENTS_GNOSIS_USDC: &str = "nep141:gnosis-0x2a22f9c3b484c3629090feed35f17ff8f88f76f0.omft.near";
pub const NEAR_INTENTS_APT_NATIVE: &str = "nep141:aptos.omft.near";
pub const NEAR_INTENTS_ZEC_NATIVE: &str = "nep141:zec.omft.near";
#[allow(unused)]
pub const NEAR_INTENTS_STELLAR_NATIVE: &str = "nep245:v2_1.omni.hot.tg:1100_111bzQBB5v7AhLyPMDwS8uJgQV24KaAPXtwyVWu2KXbbfQU6NXRCz";

type AssetsMap = HashMap<String, &'static str>;

pub static NEAR_INTENTS_ASSETS: LazyLock<HashMap<Chain, AssetsMap>> = LazyLock::new(|| {
    let mut map: HashMap<Chain, AssetsMap> = HashMap::new();

    map.insert(Chain::Near, HashMap::from([("near".to_string(), NEAR_INTENTS_WRAP_NEAR)]));

    map.insert(
        Chain::Ethereum,
        HashMap::from([
            ("ethereum".to_string(), NEAR_INTENTS_ETH_NATIVE),
            (asset_key(USDC_ETH_ASSET_ID), NEAR_INTENTS_ETH_USDC),
            (asset_key(USDT_ETH_ASSET_ID), NEAR_INTENTS_ETH_USDT),
        ]),
    );

    map.insert(Chain::Bitcoin, HashMap::from([("bitcoin".to_string(), NEAR_INTENTS_BTC_NATIVE)]));

    map.insert(
        Chain::Solana,
        HashMap::from([
            ("solana".to_string(), NEAR_INTENTS_SOL_NATIVE),
            (asset_key(USDC_SOLANA_ASSET_ID), NEAR_INTENTS_SOL_USDC),
            (asset_key(USDT_SOLANA_ASSET_ID), NEAR_INTENTS_SOL_USDT),
        ]),
    );

    map.insert(Chain::Sui, HashMap::from([("sui".to_string(), NEAR_INTENTS_SUI_NATIVE)]));

    map.insert(
        Chain::Arbitrum,
        HashMap::from([
            ("arbitrum".to_string(), NEAR_INTENTS_ARB_NATIVE),
            (asset_key(USDC_ARB_ASSET_ID), NEAR_INTENTS_ARB_USDC),
            (asset_key(USDT_ARB_ASSET_ID), NEAR_INTENTS_ARB_USDT),
        ]),
    );

    map.insert(
        Chain::Base,
        HashMap::from([
            ("base".to_string(), NEAR_INTENTS_BASE_NATIVE),
            (asset_key(USDC_BASE_ASSET_ID), NEAR_INTENTS_BASE_USDC),
        ]),
    );

    map.insert(
        Chain::Optimism,
        HashMap::from([
            ("optimism".to_string(), NEAR_INTENTS_OPT_NATIVE),
            (asset_key(USDC_OP_ASSET_ID), NEAR_INTENTS_OPT_USDC),
            (asset_key(USDT_OP_ASSET_ID), NEAR_INTENTS_OPT_USDT),
        ]),
    );

    map.insert(
        Chain::AvalancheC,
        HashMap::from([
            ("avalanchec".to_string(), NEAR_INTENTS_AVAX_NATIVE),
            (asset_key(USDC_AVAX_ASSET_ID), NEAR_INTENTS_AVAX_USDC),
            (asset_key(USDT_AVAX_ASSET_ID), NEAR_INTENTS_AVAX_USDT),
        ]),
    );

    map.insert(
        Chain::SmartChain,
        HashMap::from([
            ("smartchain".to_string(), NEAR_INTENTS_BSC_NATIVE),
            (asset_key(USDT_SMARTCHAIN_ASSET_ID), NEAR_INTENTS_BSC_USDT),
        ]),
    );

    map.insert(
        Chain::Polygon,
        HashMap::from([
            ("polygon".to_string(), NEAR_INTENTS_POL_NATIVE),
            (asset_key(USDC_POLYGON_ASSET_ID), NEAR_INTENTS_POL_USDC),
            (asset_key(USDT_POLYGON_ASSET_ID), NEAR_INTENTS_POL_USDT),
        ]),
    );

    map.insert(
        Chain::Ton,
        HashMap::from([
            ("ton".to_string(), NEAR_INTENTS_TON_NATIVE),
            (asset_key(USDT_TON_ASSET_ID), NEAR_INTENTS_TON_USDT),
        ]),
    );

    map.insert(
        Chain::Tron,
        HashMap::from([
            ("tron".to_string(), NEAR_INTENTS_TRON_NATIVE),
            (asset_key(USDT_TRON_ASSET_ID), NEAR_INTENTS_TRON_USDT),
        ]),
    );

    map.insert(Chain::Doge, HashMap::from([("doge".to_string(), NEAR_INTENTS_DOGE_NATIVE)]));
    map.insert(Chain::Xrp, HashMap::from([("xrp".to_string(), NEAR_INTENTS_XRP_NATIVE)]));
    map.insert(Chain::Cardano, HashMap::from([("cardano".to_string(), NEAR_INTENTS_CARDANO_NATIVE)]));
    map.insert(Chain::Berachain, HashMap::from([("berachain".to_string(), NEAR_INTENTS_BERA_NATIVE)]));
    map.insert(Chain::Aptos, HashMap::from([("aptos".to_string(), NEAR_INTENTS_APT_NATIVE)]));
    map.insert(Chain::Zcash, HashMap::from([("zcash".to_string(), NEAR_INTENTS_ZEC_NATIVE)]));

    map.insert(
        Chain::Gnosis,
        HashMap::from([
            ("gnosis".to_string(), NEAR_INTENTS_GNOSIS_NATIVE),
            (asset_key(USDC_GNOSIS_ASSET_ID), NEAR_INTENTS_GNOSIS_USDC),
        ]),
    );

    map
});

fn asset_key(asset_id: &str) -> String {
    asset_id.to_ascii_lowercase()
}

pub static NEAR_INTENTS_REVERSE_ASSETS: LazyLock<HashMap<&'static str, AssetId>> = LazyLock::new(|| {
    let mut reverse = HashMap::new();
    for (chain, assets) in NEAR_INTENTS_ASSETS.iter() {
        for (asset_key, near_asset) in assets.iter() {
            match AssetId::new(asset_key) {
                Some(asset_id) => {
                    reverse.insert(*near_asset, asset_id);
                }
                None => {
                    reverse.insert(*near_asset, AssetId::from_chain(*chain));
                }
            }
        }
    }
    reverse
});

pub fn get_near_intents_asset_id(asset: &SwapperQuoteAsset) -> Result<String, SwapperError> {
    let asset_id = asset.asset_id();
    let key = asset_id.to_string().to_lowercase();
    let chain_assets = NEAR_INTENTS_ASSETS.get(&asset_id.chain).ok_or(SwapperError::NotSupportedChain)?;

    chain_assets.get(&key).map(|value| value.to_string()).ok_or(SwapperError::NotSupportedAsset)
}

pub fn asset_id_from_near_intents(near_asset: &str) -> Option<AssetId> {
    NEAR_INTENTS_REVERSE_ASSETS.get(near_asset).cloned()
}

pub fn supported_assets() -> Vec<SwapperChainAsset> {
    NEAR_INTENTS_ASSETS
        .iter()
        .map(|(chain, assets)| {
            let asset_ids = assets.keys().filter_map(|value| AssetId::new(value)).collect::<Vec<_>>();
            SwapperChainAsset::Assets(*chain, asset_ids)
        })
        .collect()
}

pub fn enabled_sending_chains() -> Vec<Chain> {
    // TODO: reivew other chains provider/preload for fee estimation before adding new chains
    vec![Chain::Near, Chain::Ethereum, Chain::Sui, Chain::SmartChain, Chain::Doge]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_asset_id() {
        let asset = SwapperQuoteAsset {
            id: "ethereum_0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48".into(),
            symbol: "USDC".into(),
            decimals: 6,
        };

        let result = get_near_intents_asset_id(&asset).unwrap();
        assert_eq!(result, NEAR_INTENTS_ETH_USDC);
    }

    #[test]
    fn test_supported_assets_contains_near() {
        let supported = supported_assets();
        assert!(
            supported
                .iter()
                .any(|entry| matches!(entry, SwapperChainAsset::Assets(chain, _) if *chain == Chain::Near))
        );
    }

    #[test]
    fn test_asset_id_from_near_intents() {
        let asset = asset_id_from_near_intents(NEAR_INTENTS_ETH_USDC).unwrap();
        assert_eq!(asset.chain, Chain::Ethereum);
    }
}
