use crate::chain::Chain;
use crate::chain_evm::EVMChain;
use crate::explorers::{
    AlgorandAllo, BlockScout, Blocksec, Cardanocan, EtherScan, HyperliquidExplorer, MantleExplorer, NearBlocks, OkxExplorer, RouteScan, RuneScan, SubScan,
    TonScan, TronScan, Viewblock, XrpScan, ZkSync, aptos, blockchair, mempool, mintscan, solana, stellar_expert, sui, ton,
};
use std::str::FromStr;
use typeshare::typeshare;

pub trait BlockExplorer: Send + Sync {
    fn name(&self) -> String;
    fn get_tx_url(&self, hash: &str) -> String;
    fn get_address_url(&self, address: &str) -> String;
    fn get_token_url(&self, _token: &str) -> Option<String> {
        None
    }
    fn get_validator_url(&self, _validator: &str) -> Option<String> {
        None
    }

    fn new() -> Box<Self>
    where
        Self: Default + Sized,
    {
        Box::new(Self::default())
    }
}

#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[allow(dead_code)]
struct BlockExplorerLink {
    name: String,
    link: String,
}

pub fn get_block_explorers_by_chain(chain: &str) -> Vec<Box<dyn BlockExplorer>> {
    let Ok(chain) = Chain::from_str(chain) else {
        return vec![];
    };
    get_block_explorers(chain)
}

pub fn get_block_explorer(chain: Chain, name: &str) -> Box<dyn BlockExplorer> {
    get_block_explorers(chain).into_iter().find(|x| x.name() == name).unwrap()
}

pub fn get_block_explorers(chain: Chain) -> Vec<Box<dyn BlockExplorer>> {
    match chain {
        Chain::Bitcoin => vec![blockchair::new_bitcoin(), mempool::new()],
        Chain::BitcoinCash => vec![blockchair::new_bitcoin_cash()],
        Chain::Litecoin => vec![blockchair::new_litecoin()],
        Chain::Doge => vec![blockchair::new_doge()],
        Chain::Zcash => vec![blockchair::new_zcash()],

        Chain::Ethereum => vec![EtherScan::boxed(EVMChain::Ethereum), blockchair::new_ethereum(), Blocksec::new_ethereum()],
        Chain::SmartChain => vec![EtherScan::boxed(EVMChain::SmartChain), blockchair::new_bnb(), Blocksec::new_bsc()],
        Chain::Polygon => vec![EtherScan::boxed(EVMChain::Polygon), blockchair::new_polygon(), Blocksec::new_polygon()],
        Chain::Arbitrum => vec![EtherScan::boxed(EVMChain::Arbitrum), blockchair::new_arbitrum(), Blocksec::new_arbitrum()],
        Chain::Optimism => vec![EtherScan::boxed(EVMChain::Optimism), blockchair::new_optimism(), Blocksec::new_optimism()],
        Chain::Base => vec![EtherScan::boxed(EVMChain::Base), blockchair::new_base(), Blocksec::new_base()],
        Chain::AvalancheC => vec![EtherScan::boxed(EVMChain::AvalancheC), RouteScan::new_avax(), blockchair::new_avalanche()],
        Chain::OpBNB => vec![EtherScan::boxed(EVMChain::OpBNB), blockchair::new_opbnb()],
        Chain::Fantom => vec![EtherScan::boxed(EVMChain::Fantom), blockchair::new_fantom()],
        Chain::Gnosis => vec![EtherScan::boxed(EVMChain::Gnosis), blockchair::new_gnosis()],
        Chain::Manta => vec![BlockScout::new_manta(), EtherScan::boxed(EVMChain::Manta)],
        Chain::Blast => vec![EtherScan::boxed(EVMChain::Blast)],
        Chain::Linea => vec![EtherScan::boxed(EVMChain::Linea), blockchair::new_linea()],
        Chain::Celo => vec![BlockScout::new_celo(), EtherScan::boxed(EVMChain::Celo)],
        Chain::ZkSync => vec![ZkSync::boxed(), EtherScan::boxed(EVMChain::ZkSync)],
        Chain::World => vec![EtherScan::boxed(EVMChain::World)],
        Chain::Plasma => vec![EtherScan::boxed(EVMChain::Plasma)],
        Chain::Solana => vec![solana::new_solscan(), solana::new_solana_fm(), blockchair::new_solana()],
        Chain::Thorchain => vec![RuneScan::boxed(), Viewblock::boxed()],

        Chain::Cosmos => vec![mintscan::new_cosmos()],
        Chain::Osmosis => vec![mintscan::new_osmosis()],
        Chain::Celestia => vec![mintscan::new_celestia()],
        Chain::Injective => vec![mintscan::new_injective()],
        Chain::Sei => vec![mintscan::new_sei()],
        Chain::Noble => vec![mintscan::new_noble()],
        Chain::Mantle => vec![MantleExplorer::boxed(), EtherScan::boxed(EVMChain::Mantle)],

        Chain::Ton => vec![ton::new_ton_viewer(), TonScan::boxed(), blockchair::new_ton()],
        Chain::Tron => vec![TronScan::boxed(), blockchair::new_tron()],
        Chain::Xrp => vec![XrpScan::boxed(), blockchair::new_xrp()],
        Chain::Aptos => vec![aptos::new_aptos_scan(), aptos::new_aptos_explorer(), blockchair::new_aptos()],
        Chain::Sui => vec![sui::new_sui_scan(), sui::new_sui_vision()],
        Chain::Near => vec![NearBlocks::boxed()],
        Chain::Stellar => vec![stellar_expert::new(), blockchair::new_stellar()],
        Chain::Sonic => vec![EtherScan::boxed(EVMChain::Sonic), RouteScan::new_sonic()],
        Chain::Algorand => vec![AlgorandAllo::boxed()],
        Chain::Polkadot => vec![SubScan::new_polkadot(), blockchair::new_polkadot()],
        Chain::Cardano => vec![Cardanocan::boxed()],
        Chain::Abstract => vec![EtherScan::boxed(EVMChain::Abstract)],
        Chain::Berachain => vec![EtherScan::boxed(EVMChain::Berachain)],
        Chain::Ink => vec![RouteScan::new_ink(), BlockScout::new_ink(), OkxExplorer::new_ink()],
        Chain::Unichain => vec![EtherScan::boxed(EVMChain::Unichain)],
        Chain::Hyperliquid => vec![EtherScan::boxed(EVMChain::Hyperliquid), BlockScout::new_hyperliquid()],
        Chain::HyperCore => vec![HyperliquidExplorer::boxed()],
        Chain::Monad => vec![EtherScan::boxed(EVMChain::Monad)],
    }
}
