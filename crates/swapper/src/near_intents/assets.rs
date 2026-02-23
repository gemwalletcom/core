use crate::{SwapperError, SwapperQuoteAsset, models::SwapperChainAsset};
use primitives::{
    AssetId, Chain,
    asset_constants::{
        AAVE_ETH_ASSET_ID, ARB_ARB_ASSET_ID, CBBTC_BASE_ASSET_ID, CBBTC_ETH_ASSET_ID, DAI_ETH_ASSET_ID, LINK_ETH_ASSET_ID, OP_OP_ASSET_ID, UNI_ETH_ASSET_ID, USDC_ARB_ASSET_ID,
        USDC_AVAX_ASSET_ID, USDC_BASE_ASSET_ID, USDC_ETH_ASSET_ID, USDC_GNOSIS_ASSET_ID, USDC_MONAD_ASSET_ID, USDC_OP_ASSET_ID, USDC_POLYGON_ASSET_ID, USDC_SMARTCHAIN_ASSET_ID,
        USDC_SOLANA_ASSET_ID, USDC_SUI_ASSET_ID, USDC_XLAYER_ASSET_ID, USDT_APTOS_ASSET_ID, USDT_ARB_ASSET_ID, USDT_AVAX_ASSET_ID, USDT_BERA_ASSET_ID,
        USDT_ETH_ASSET_ID, USDT_GNOSIS_ASSET_ID, USDT_MONAD_ASSET_ID, USDT_OP_ASSET_ID, USDT_PLASMA_ASSET_ID, USDT_POLYGON_ASSET_ID, USDT_SMARTCHAIN_ASSET_ID,
        USDT_SOLANA_ASSET_ID, USDT_TON_ASSET_ID, USDT_TRON_ASSET_ID, USDT_XLAYER_ASSET_ID, WBTC_ETH_ASSET_ID,
    },
};
use std::{collections::HashMap, sync::LazyLock};

pub const NEAR_INTENTS_WRAP_NEAR: &str = "nep141:wrap.near";
// pub const NEAR_INTENTS_NEAR_USDC: &str = "nep141:17208628f84f5d6ad33f0da3bbbeb27ffcb398eac501a31bd6ad2011e36133a1";
// pub const NEAR_INTENTS_NEAR_USDT: &str = "nep141:usdt.tether-token.near";
pub const NEAR_INTENTS_ETH_NATIVE: &str = "nep141:eth.omft.near";
pub const NEAR_INTENTS_ETH_USDC: &str = "nep141:eth-0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48.omft.near";
pub const NEAR_INTENTS_ETH_USDT: &str = "nep141:eth-0xdac17f958d2ee523a2206206994597c13d831ec7.omft.near";
pub const NEAR_INTENTS_ETH_WBTC: &str = "nep141:eth-0x2260fac5e5542a773aa44fbcfedf7c193bc2c599.omft.near";
pub const NEAR_INTENTS_ETH_DAI: &str = "nep141:eth-0x6b175474e89094c44da98b954eedeac495271d0f.omft.near";
pub const NEAR_INTENTS_ETH_CBBTC: &str = "nep141:eth-0xcbb7c0000ab88b473b1f5afd9ef808440eed33bf.omft.near";
pub const NEAR_INTENTS_ETH_LINK: &str = "nep141:eth-0x514910771af9ca656af840dff83e8264ecf986ca.omft.near";
pub const NEAR_INTENTS_ETH_UNI: &str = "nep141:eth-0x1f9840a85d5af5bf1d1762f925bdaddc4201f984.omft.near";
pub const NEAR_INTENTS_ETH_AAVE: &str = "nep141:eth-0x7fc66500c84a76ad7e9c93437bfc5ac33e2ddae9.omft.near";
pub const NEAR_INTENTS_BTC_NATIVE: &str = "nep141:btc.omft.near";
pub const NEAR_INTENTS_SOL_NATIVE: &str = "nep141:sol.omft.near";
pub const NEAR_INTENTS_SOL_USDC: &str = "nep141:sol-5ce3bf3a31af18be40ba30f721101b4341690186.omft.near";
pub const NEAR_INTENTS_SOL_USDT: &str = "nep141:sol-c800a4bd850783ccb82c2b2c7e84175443606352.omft.near";
pub const NEAR_INTENTS_SUI_NATIVE: &str = "nep141:sui.omft.near";
pub const NEAR_INTENTS_SUI_USDC: &str = "nep141:sui-c1b81ecaf27933252d31a963bc5e9458f13c18ce.omft.near";
pub const NEAR_INTENTS_ARB_NATIVE: &str = "nep141:arb.omft.near";
pub const NEAR_INTENTS_ARB_USDC: &str = "nep141:arb-0xaf88d065e77c8cc2239327c5edb3a432268e5831.omft.near";
pub const NEAR_INTENTS_ARB_USDT: &str = "nep141:arb-0xfd086bc7cd5c481dcc9c85ebe478a1c0b69fcbb9.omft.near";
pub const NEAR_INTENTS_ARB_ARB: &str = "nep141:arb-0x912ce59144191c1204e64559fe8253a0e49e6548.omft.near";
pub const NEAR_INTENTS_BASE_NATIVE: &str = "nep141:base.omft.near";
pub const NEAR_INTENTS_BASE_USDC: &str = "nep141:base-0x833589fcd6edb6e08f4c7c32d4f71b54bda02913.omft.near";
pub const NEAR_INTENTS_BASE_CBBTC: &str = "nep141:base-0xcbb7c0000ab88b473b1f5afd9ef808440eed33bf.omft.near";
pub const NEAR_INTENTS_OPT_NATIVE: &str = "nep245:v2_1.omni.hot.tg:10_11111111111111111111";
pub const NEAR_INTENTS_OPT_USDC: &str = "nep245:v2_1.omni.hot.tg:10_A2ewyUyDp6qsue1jqZsGypkCxRJ";
pub const NEAR_INTENTS_OPT_USDT: &str = "nep245:v2_1.omni.hot.tg:10_359RPSJVdTxwTJT9TyGssr2rFoWo";
pub const NEAR_INTENTS_OPT_OP: &str = "nep245:v2_1.omni.hot.tg:10_vLAiSt9KfUGKpw5cD3vsSyNYBo7";
pub const NEAR_INTENTS_AVAX_NATIVE: &str = "nep245:v2_1.omni.hot.tg:43114_11111111111111111111";
pub const NEAR_INTENTS_AVAX_USDC: &str = "nep245:v2_1.omni.hot.tg:43114_3atVJH3r5c4GqiSYmg9fECvjc47o";
pub const NEAR_INTENTS_AVAX_USDT: &str = "nep245:v2_1.omni.hot.tg:43114_372BeH7ENZieCaabwkbWkBiTTgXp";
pub const NEAR_INTENTS_BSC_NATIVE: &str = "nep245:v2_1.omni.hot.tg:56_11111111111111111111";
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
pub const NEAR_INTENTS_APT_USDT: &str = "nep141:aptos-88cb7619440a914fe6400149a12b443c3ac21d59.omft.near";
pub const NEAR_INTENTS_ZEC_NATIVE: &str = "nep141:zec.omft.near";
pub const NEAR_INTENTS_STELLAR_NATIVE: &str = "nep245:v2_1.omni.hot.tg:1100_111bzQBB5v7AhLyPMDwS8uJgQV24KaAPXtwyVWu2KXbbfQU6NXRCz";
// pub const NEAR_INTENTS_STELLAR_USDC: &str = "nep245:v2_1.omni.hot.tg:1100_111bzQBB65GxAPAVoxqmMcgYo5oS3txhqs1Uh1cgahKQUeTUq1TJu";
pub const NEAR_INTENTS_LTC_NATIVE: &str = "nep141:ltc.omft.near";
pub const NEAR_INTENTS_BCH_NATIVE: &str = "nep141:bch.omft.near";
pub const NEAR_INTENTS_BERA_USDT: &str = "nep141:bera-0x779ded0c9e1022225f8e0630b35a9b54be713736.omft.near";
pub const NEAR_INTENTS_GNOSIS_USDT: &str = "nep141:gnosis-0x4ecaba5870353805a9f068101a40e0f32ed605c6.omft.near";
pub const NEAR_INTENTS_MONAD_NATIVE: &str = "nep245:v2_1.omni.hot.tg:143_11111111111111111111";
pub const NEAR_INTENTS_MONAD_USDT: &str = "nep245:v2_1.omni.hot.tg:143_4EJiJxSALvGoTZbnc8K7Ft9533et";
pub const NEAR_INTENTS_MONAD_USDC: &str = "nep245:v2_1.omni.hot.tg:143_2dmLwYWkCQKyTjeUPAsGJuiVLbFx";
pub const NEAR_INTENTS_XLAYER_NATIVE: &str = "nep245:v2_1.omni.hot.tg:196_11111111111111111111";
pub const NEAR_INTENTS_XLAYER_USDT: &str = "nep245:v2_1.omni.hot.tg:196_2fezDCvVYRsG8wrK6deJ2VRPiAS1";
pub const NEAR_INTENTS_XLAYER_USDC: &str = "nep245:v2_1.omni.hot.tg:196_2dK9kLNR7Ekq7su8FxNGiUW3djTw";
pub const NEAR_INTENTS_PLASMA_NATIVE: &str = "nep245:v2_1.omni.hot.tg:9745_11111111111111111111";
pub const NEAR_INTENTS_PLASMA_USDT: &str = "nep245:v2_1.omni.hot.tg:9745_3aL9skCy1yhPoDB8oKMmRHRN7SJW";

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
            (asset_key(WBTC_ETH_ASSET_ID), NEAR_INTENTS_ETH_WBTC),
            (asset_key(DAI_ETH_ASSET_ID), NEAR_INTENTS_ETH_DAI),
            (asset_key(CBBTC_ETH_ASSET_ID), NEAR_INTENTS_ETH_CBBTC),
            (asset_key(LINK_ETH_ASSET_ID), NEAR_INTENTS_ETH_LINK),
            (asset_key(UNI_ETH_ASSET_ID), NEAR_INTENTS_ETH_UNI),
            (asset_key(AAVE_ETH_ASSET_ID), NEAR_INTENTS_ETH_AAVE),
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

    map.insert(
        Chain::Sui,
        HashMap::from([("sui".to_string(), NEAR_INTENTS_SUI_NATIVE), (asset_key(USDC_SUI_ASSET_ID), NEAR_INTENTS_SUI_USDC)]),
    );

    map.insert(
        Chain::Arbitrum,
        HashMap::from([
            ("arbitrum".to_string(), NEAR_INTENTS_ARB_NATIVE),
            (asset_key(USDC_ARB_ASSET_ID), NEAR_INTENTS_ARB_USDC),
            (asset_key(USDT_ARB_ASSET_ID), NEAR_INTENTS_ARB_USDT),
            (asset_key(ARB_ARB_ASSET_ID), NEAR_INTENTS_ARB_ARB),
        ]),
    );

    map.insert(
        Chain::Base,
        HashMap::from([
            ("base".to_string(), NEAR_INTENTS_BASE_NATIVE),
            (asset_key(USDC_BASE_ASSET_ID), NEAR_INTENTS_BASE_USDC),
            (asset_key(CBBTC_BASE_ASSET_ID), NEAR_INTENTS_BASE_CBBTC),
        ]),
    );

    map.insert(
        Chain::Optimism,
        HashMap::from([
            ("optimism".to_string(), NEAR_INTENTS_OPT_NATIVE),
            (asset_key(USDC_OP_ASSET_ID), NEAR_INTENTS_OPT_USDC),
            (asset_key(USDT_OP_ASSET_ID), NEAR_INTENTS_OPT_USDT),
            (asset_key(OP_OP_ASSET_ID), NEAR_INTENTS_OPT_OP),
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
            (asset_key(USDC_SMARTCHAIN_ASSET_ID), NEAR_INTENTS_BSC_USDC),
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
        HashMap::from([("ton".to_string(), NEAR_INTENTS_TON_NATIVE), (asset_key(USDT_TON_ASSET_ID), NEAR_INTENTS_TON_USDT)]),
    );

    map.insert(
        Chain::Tron,
        HashMap::from([("tron".to_string(), NEAR_INTENTS_TRON_NATIVE), (asset_key(USDT_TRON_ASSET_ID), NEAR_INTENTS_TRON_USDT)]),
    );

    map.insert(Chain::Doge, HashMap::from([("doge".to_string(), NEAR_INTENTS_DOGE_NATIVE)]));
    map.insert(Chain::Xrp, HashMap::from([("xrp".to_string(), NEAR_INTENTS_XRP_NATIVE)]));
    map.insert(Chain::Cardano, HashMap::from([("cardano".to_string(), NEAR_INTENTS_CARDANO_NATIVE)]));
    map.insert(
        Chain::Berachain,
        HashMap::from([("berachain".to_string(), NEAR_INTENTS_BERA_NATIVE), (asset_key(USDT_BERA_ASSET_ID), NEAR_INTENTS_BERA_USDT)]),
    );
    map.insert(
        Chain::Aptos,
        HashMap::from([("aptos".to_string(), NEAR_INTENTS_APT_NATIVE), (asset_key(USDT_APTOS_ASSET_ID), NEAR_INTENTS_APT_USDT)]),
    );
    map.insert(Chain::Zcash, HashMap::from([("zcash".to_string(), NEAR_INTENTS_ZEC_NATIVE)]));

    map.insert(
        Chain::Gnosis,
        HashMap::from([
            ("gnosis".to_string(), NEAR_INTENTS_GNOSIS_NATIVE),
            (asset_key(USDC_GNOSIS_ASSET_ID), NEAR_INTENTS_GNOSIS_USDC),
            (asset_key(USDT_GNOSIS_ASSET_ID), NEAR_INTENTS_GNOSIS_USDT),
        ]),
    );

    map.insert(Chain::Stellar, HashMap::from([("stellar".to_string(), NEAR_INTENTS_STELLAR_NATIVE)]));

    map.insert(Chain::Litecoin, HashMap::from([("litecoin".to_string(), NEAR_INTENTS_LTC_NATIVE)]));
    map.insert(Chain::BitcoinCash, HashMap::from([("bitcoincash".to_string(), NEAR_INTENTS_BCH_NATIVE)]));

    map.insert(
        Chain::Monad,
        HashMap::from([
            ("monad".to_string(), NEAR_INTENTS_MONAD_NATIVE),
            (asset_key(USDT_MONAD_ASSET_ID), NEAR_INTENTS_MONAD_USDT),
            (asset_key(USDC_MONAD_ASSET_ID), NEAR_INTENTS_MONAD_USDC),
        ]),
    );

    map.insert(
        Chain::XLayer,
        HashMap::from([
            ("xlayer".to_string(), NEAR_INTENTS_XLAYER_NATIVE),
            (asset_key(USDT_XLAYER_ASSET_ID), NEAR_INTENTS_XLAYER_USDT),
            (asset_key(USDC_XLAYER_ASSET_ID), NEAR_INTENTS_XLAYER_USDC),
        ]),
    );

    map.insert(
        Chain::Plasma,
        HashMap::from([
            ("plasma".to_string(), NEAR_INTENTS_PLASMA_NATIVE),
            (asset_key(USDT_PLASMA_ASSET_ID), NEAR_INTENTS_PLASMA_USDT),
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
            let asset_ids = assets.keys().filter_map(|x| AssetId::new(x)).collect::<Vec<_>>();
            SwapperChainAsset::Assets(*chain, asset_ids)
        })
        .collect()
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
        assert!(supported.iter().any(|entry| matches!(entry, SwapperChainAsset::Assets(chain, _) if *chain == Chain::Near)));
    }

    #[test]
    fn test_asset_id_from_near_intents() {
        let asset = asset_id_from_near_intents(NEAR_INTENTS_ETH_USDC).unwrap();
        assert_eq!(asset.chain, Chain::Ethereum);
    }
}
