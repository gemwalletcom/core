use crate::chain::Chain;
use crate::chain_evm::EVMChain;
use crate::explorers::{
    AptosExplorer, AptosScan, BlockScout, Blockchair, Blocksec, EtherScan, MantleExplorer, Mempool, MintScan, NearBlocks, RuneScan, SolanaFM, Solscan, SuiScan,
    SuiVision, TonScan, TonViewer, TronScan, Viewblock, XrpScan, ZkSync,
};
use std::str::FromStr;
use typeshare::typeshare;

pub trait BlockExplorer: Send + Sync {
    fn name(&self) -> String;
    fn get_tx_url(&self, hash: &str) -> String;
    fn get_address_url(&self, address: &str) -> String;
    fn get_token_url(&self, token: &str) -> Option<String>;
    fn get_validator_url(&self, validator: &str) -> Option<String>;
}
pub struct Metadata {
    pub name: &'static str,
    pub base_url: &'static str,
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
        Chain::Bitcoin => vec![Blockchair::new_bitcoin(), Mempool::new()],
        Chain::Litecoin => vec![Blockchair::new_litecoin()],
        Chain::Doge => vec![Blockchair::new_doge()],

        Chain::Ethereum => vec![EtherScan::new(EVMChain::Ethereum), Blockchair::new_ethereum(), Blocksec::new_ethereum()],
        Chain::SmartChain => vec![EtherScan::new(EVMChain::SmartChain), Blockchair::new_bnb(), Blocksec::new_bsc()],
        Chain::Polygon => vec![EtherScan::new(EVMChain::Polygon), Blockchair::new_polygon(), Blocksec::new_polygon()],
        Chain::Arbitrum => vec![EtherScan::new(EVMChain::Arbitrum), Blockchair::new_arbitrum(), Blocksec::new_arbitrum()],
        Chain::Optimism => vec![EtherScan::new(EVMChain::Optimism), Blockchair::new_optimism(), Blocksec::new_optimism()],
        Chain::Base => vec![EtherScan::new(EVMChain::Base), Blockchair::new_base(), Blocksec::new_ethereum()],
        Chain::AvalancheC => vec![EtherScan::new(EVMChain::AvalancheC), Blockchair::new_avalanche()],
        Chain::OpBNB => vec![EtherScan::new(EVMChain::OpBNB), Blockchair::new_opbnb()],
        Chain::Fantom => vec![EtherScan::new(EVMChain::Fantom), Blockchair::new_fantom()],
        Chain::Gnosis => vec![EtherScan::new(EVMChain::Gnosis), Blockchair::new_gnosis()],
        Chain::Manta => vec![BlockScout::new_manta(), EtherScan::new(EVMChain::Manta)],
        Chain::Blast => vec![EtherScan::new(EVMChain::Blast)],
        Chain::Linea => vec![EtherScan::new(EVMChain::Linea), Blockchair::new_linea()],
        Chain::Celo => vec![BlockScout::new_celo(), EtherScan::new(EVMChain::Celo)],
        Chain::ZkSync => vec![ZkSync::new(), EtherScan::new(EVMChain::ZkSync)],
        Chain::World => vec![EtherScan::new(EVMChain::World)],
        Chain::Solana => vec![SolanaFM::new(), Solscan::new(), Blockchair::new_solana()],
        Chain::Thorchain => vec![Viewblock::new(), RuneScan::new()],

        Chain::Cosmos => vec![MintScan::new_cosmos()],
        Chain::Osmosis => vec![MintScan::new_osmosis()],
        Chain::Celestia => vec![MintScan::new_celestia()],
        Chain::Injective => vec![MintScan::new_injective()],
        Chain::Sei => vec![MintScan::new_sei()],
        Chain::Noble => vec![MintScan::new_noble()],
        Chain::Mantle => vec![MantleExplorer::new(), EtherScan::new(EVMChain::Mantle)],

        Chain::Ton => vec![TonViewer::new(), TonScan::new(), Blockchair::new_ton()],
        Chain::Tron => vec![TronScan::new(), Blockchair::new_tron()],
        Chain::Xrp => vec![XrpScan::new(), Blockchair::new_xrp()],
        Chain::Aptos => vec![AptosExplorer::new(), AptosScan::new(), Blockchair::new_aptos()],
        Chain::Sui => vec![SuiScan::new(), SuiVision::new()],
        Chain::Near => vec![NearBlocks::new()],
        Chain::Stellar => vec![Blockchair::new_stellar()],
    }
}
