use primitives::{AssetId, Chain};

pub fn asset_id_for_feed_id(feed_id: &str) -> Option<AssetId> {
    chain_for_feed_id(feed_id).map(|c| c.as_asset_id())
}

fn chain_for_feed_id(feed_id: &str) -> Option<Chain> {
    match feed_id {
        "e62df6c8b4a85fe1a67db44dc12de5db330f7ac66b72dc658afedf0f4a415b43" => Some(Chain::Bitcoin),
        "3dd2b63686a450ec7290df3a1e0b583c0481f651351edfa7636f39aed55cf8a3" => Some(Chain::BitcoinCash),
        "6e3f3fa8253588df9326580180233eb791e03b443a3ba7a1d892e73874e19a54" => Some(Chain::Litecoin),
        "ff61491a931112ddf1bd8147cd1b641375f79f5825126d665480874634fd0ace" => Some(Chain::Ethereum),
        "2f95862b045670cd22bee3114c39763a4a08beeb663b145d283c31d7d1101c4f" => Some(Chain::SmartChain),
        "ef0d8b6fda2ceba41da15d4095d1da392a0d2f8ed0c6c7bc0f4cfac8c280b56d" => Some(Chain::Solana),
        "5fcf71143bb70d41af4fa9aa1287e2efd3c5911cee59f909f915c9f61baacb1e" => Some(Chain::Thorchain),
        "b00b60f88b03a6a625a8d1c048c3f66653edf217439983d037e7222c4e612819" => Some(Chain::Cosmos),
        "5867f5683c757393a0670ef0f701490950fe93fdb006d181c8265a831ac0c5c6" => Some(Chain::Osmosis),
        "8963217838ab4cf5cadc172203c1f0b763fbaa45f346d8ee50ba994bbcac3026" => Some(Chain::Ton),
        "67aed5a24fdad045475e7195c98a98aea119c763f272d4523f5bac93a4f33c2b" => Some(Chain::Tron),
        "dcef50dd0a4cd2dcc17e45df1676dcb336a11a61c69df7a0299b0150c672d25c" => Some(Chain::Doge),
        "be9b59d178f0d6a97ab4c343bff2aa69caa1eaae3e9048a65788c529b125bb24" => Some(Chain::Zcash),
        "03ae4db29ed4ae33d323568895aa00337e658e348b37509f5372ae51f0af00d5" => Some(Chain::Aptos),
        "93da3352f9f1d105fdfe4971cfa80e9dd777bfc5d0f683ebb6e1294b92137bb7" => Some(Chain::AvalancheC),
        "23d7315113f5b1d3ba7a83604c44b94d79f4fd69af77f804fc7f920a6dc65744" => Some(Chain::Sui),
        "ec5d399846a9209f3fe5881d70aae9268c94339ff9817e8d18ff19fa05eea1c8" => Some(Chain::Xrp),
        "09f7c1d7dfbb7df2b8fe3d3d87ee94a2259d212da4f30c1f0540d066dfa44723" => Some(Chain::Celestia),
        "7a5bc1d2b56ad029048cd63964b3ad2776eadf812edc1a43a31406cb54bff592" => Some(Chain::Injective),
        "53614f1cb0c031d4af66c04cb9c756234adad0e1cee85303795091499a4084eb" => Some(Chain::Sei),
        "4e3037c822d852d79af3ac80e35eb420ee3b870dca49f9344a38ef4773fb0585" => Some(Chain::Mantle),
        "7d669ddcdd23d9ef1fa9a9cc022ba055ec900e91c4cb960f3c20429d4447a411" => Some(Chain::Celo),
        "c415de8d2eba7db216527dff4b60e8f3a5311c740dadb233e13e12547e226750" => Some(Chain::Near),
        "b7a8eba68a997cd0210c2e1e4ee811ad2d174b3611c22d9ebf16f4cb7e9ba850" => Some(Chain::Stellar),
        "fa17ceaf30d19ba51112fdcc750cc83454776f47fb0112e4af07f15f4bb1ebc0" => Some(Chain::Algorand),
        "ca3eed9b267293f6595901c734c7525ce8ef49adafe8284606ceb307afa2ca5b" => Some(Chain::Polkadot),
        "2a01deaec9e51a579277b34b122399984d0bbf57e2458a7e42fecd2829867a0d" => Some(Chain::Cardano),
        "4279e31cc369bbcc2faf022b382b080e32a8e689ff20fbc530d2a603eb6cd98b" => Some(Chain::Hyperliquid),
        "c5f60d00d926ee369ded32a38a6bd5c1e0faa936f91b987a5d0dcf3c5d8afab0" => Some(Chain::Gnosis),
        _ => None,
    }
}

// https://www.pyth.network/price-feeds
// Hermes API feed IDs for each chain's native asset
pub fn price_feed_id_for_chain(chain: Chain) -> &'static str {
    match chain {
        Chain::Bitcoin => "e62df6c8b4a85fe1a67db44dc12de5db330f7ac66b72dc658afedf0f4a415b43",
        Chain::BitcoinCash => "3dd2b63686a450ec7290df3a1e0b583c0481f651351edfa7636f39aed55cf8a3",
        Chain::Litecoin => "6e3f3fa8253588df9326580180233eb791e03b443a3ba7a1d892e73874e19a54",
        Chain::Ethereum
        | Chain::Arbitrum
        | Chain::Optimism
        | Chain::Base
        | Chain::Linea
        | Chain::Manta
        | Chain::ZkSync
        | Chain::Abstract
        | Chain::Ink
        | Chain::Unichain
        | Chain::Blast
        | Chain::World
        | Chain::Plasma => "ff61491a931112ddf1bd8147cd1b641375f79f5825126d665480874634fd0ace",
        Chain::SmartChain | Chain::OpBNB => "2f95862b045670cd22bee3114c39763a4a08beeb663b145d283c31d7d1101c4f",
        Chain::Solana => "ef0d8b6fda2ceba41da15d4095d1da392a0d2f8ed0c6c7bc0f4cfac8c280b56d",
        Chain::Polygon => "ff61491a931112ddf1bd8147cd1b641375f79f5825126d665480874634fd0ace",
        Chain::Thorchain => "5fcf71143bb70d41af4fa9aa1287e2efd3c5911cee59f909f915c9f61baacb1e",
        Chain::Cosmos => "b00b60f88b03a6a625a8d1c048c3f66653edf217439983d037e7222c4e612819",
        Chain::Osmosis => "5867f5683c757393a0670ef0f701490950fe93fdb006d181c8265a831ac0c5c6",
        Chain::Ton => "8963217838ab4cf5cadc172203c1f0b763fbaa45f346d8ee50ba994bbcac3026",
        Chain::Tron => "67aed5a24fdad045475e7195c98a98aea119c763f272d4523f5bac93a4f33c2b",
        Chain::Doge => "dcef50dd0a4cd2dcc17e45df1676dcb336a11a61c69df7a0299b0150c672d25c",
        Chain::Zcash => "be9b59d178f0d6a97ab4c343bff2aa69caa1eaae3e9048a65788c529b125bb24",
        Chain::Aptos => "03ae4db29ed4ae33d323568895aa00337e658e348b37509f5372ae51f0af00d5",
        Chain::AvalancheC => "93da3352f9f1d105fdfe4971cfa80e9dd777bfc5d0f683ebb6e1294b92137bb7",
        Chain::Sui => "23d7315113f5b1d3ba7a83604c44b94d79f4fd69af77f804fc7f920a6dc65744",
        Chain::Xrp => "ec5d399846a9209f3fe5881d70aae9268c94339ff9817e8d18ff19fa05eea1c8",
        Chain::Celestia => "09f7c1d7dfbb7df2b8fe3d3d87ee94a2259d212da4f30c1f0540d066dfa44723",
        Chain::Injective => "7a5bc1d2b56ad029048cd63964b3ad2776eadf812edc1a43a31406cb54bff592",
        Chain::Sei => "53614f1cb0c031d4af66c04cb9c756234adad0e1cee85303795091499a4084eb",
        Chain::Noble => "ff61491a931112ddf1bd8147cd1b641375f79f5825126d665480874634fd0ace",
        Chain::Mantle => "4e3037c822d852d79af3ac80e35eb420ee3b870dca49f9344a38ef4773fb0585",
        Chain::Celo => "7d669ddcdd23d9ef1fa9a9cc022ba055ec900e91c4cb960f3c20429d4447a411",
        Chain::Near => "c415de8d2eba7db216527dff4b60e8f3a5311c740dadb233e13e12547e226750",
        Chain::Stellar => "b7a8eba68a997cd0210c2e1e4ee811ad2d174b3611c22d9ebf16f4cb7e9ba850",
        Chain::Algorand => "fa17ceaf30d19ba51112fdcc750cc83454776f47fb0112e4af07f15f4bb1ebc0",
        Chain::Polkadot => "ca3eed9b267293f6595901c734c7525ce8ef49adafe8284606ceb307afa2ca5b",
        Chain::Cardano => "2a01deaec9e51a579277b34b122399984d0bbf57e2458a7e42fecd2829867a0d",
        Chain::Berachain => "ff61491a931112ddf1bd8147cd1b641375f79f5825126d665480874634fd0ace",
        Chain::Hyperliquid | Chain::HyperCore => "4279e31cc369bbcc2faf022b382b080e32a8e689ff20fbc530d2a603eb6cd98b",
        Chain::Fantom | Chain::Sonic => "ff61491a931112ddf1bd8147cd1b641375f79f5825126d665480874634fd0ace",
        Chain::Gnosis => "c5f60d00d926ee369ded32a38a6bd5c1e0faa936f91b987a5d0dcf3c5d8afab0",
        Chain::Monad => "ff61491a931112ddf1bd8147cd1b641375f79f5825126d665480874634fd0ace",
        Chain::XLayer => "119d83a6073a8948e0f5f7c50e31c35af07d0ce2fa4cf35c6e2a8a6d3c68ccc9", // OKB
    }
}
