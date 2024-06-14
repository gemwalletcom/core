use crate::chain::Chain;
use crate::chain_evm::EVMChain;
use crate::explorers::{
    AptosExplorer, AptosScan, BlockScout, Blockchair, EtherScan, MantleExplorer, Mempool, MintScan,
    NearBlocks, SolanaFM, Solscan, SuiScan, SuiVision, TonViewer, TronScan, Viewblock, XrpScan,
    ZkSync,
};
use std::str::FromStr;

pub trait BlockExplorer: Send + Sync {
    fn name(&self) -> String;
    fn get_tx_url(&self, hash: &str) -> String;
    fn get_address_url(&self, address: &str) -> String;
    fn get_token_url(&self, token: &str) -> Option<String>;
}
pub struct Metadata {
    pub name: &'static str,
    pub base_url: &'static str,
}

pub fn get_block_explorers_by_chain(chain: &str) -> Vec<Box<dyn BlockExplorer>> {
    let Ok(chain) = Chain::from_str(chain) else {
        return vec![];
    };
    get_block_explorers(chain)
}

pub fn get_block_explorers(chain: Chain) -> Vec<Box<dyn BlockExplorer>> {
    match chain {
        Chain::Bitcoin => vec![Blockchair::new_bitcoin(), Mempool::new()],
        Chain::Litecoin => vec![Blockchair::new_litecoin()],
        Chain::Doge => vec![Blockchair::new_doge()],

        Chain::Ethereum => vec![
            EtherScan::new(EVMChain::Ethereum),
            Blockchair::new_ethereum(),
        ],
        Chain::SmartChain => vec![EtherScan::new(EVMChain::SmartChain)],
        Chain::Polygon => vec![EtherScan::new(EVMChain::Polygon)],
        Chain::Arbitrum => vec![EtherScan::new(EVMChain::Arbitrum)],
        Chain::Optimism => vec![EtherScan::new(EVMChain::Optimism)],
        Chain::Base => vec![EtherScan::new(EVMChain::Base), Blockchair::new_base()],
        Chain::AvalancheC => vec![EtherScan::new(EVMChain::AvalancheC)],
        Chain::OpBNB => vec![EtherScan::new(EVMChain::OpBNB)],
        Chain::Fantom => vec![EtherScan::new(EVMChain::Fantom)],
        Chain::Gnosis => vec![EtherScan::new(EVMChain::Gnosis)],
        Chain::Manta => vec![EtherScan::new(EVMChain::Manta)],
        Chain::Blast => vec![EtherScan::new(EVMChain::Blast)],
        Chain::Linea => vec![EtherScan::new(EVMChain::Linea)],
        Chain::Celo => vec![BlockScout::new_celo(), EtherScan::new(EVMChain::Celo)],
        Chain::ZkSync => vec![ZkSync::new(), EtherScan::new(EVMChain::ZkSync)],
        Chain::Solana => vec![SolanaFM::new(), Solscan::new()],
        Chain::Thorchain => vec![Viewblock::new()],

        Chain::Cosmos => vec![MintScan::new_cosmos()],
        Chain::Osmosis => vec![MintScan::new_osmosis()],
        Chain::Celestia => vec![MintScan::new_celestia()],
        Chain::Injective => vec![MintScan::new_injective()],
        Chain::Sei => vec![MintScan::new_sei()],
        Chain::Noble => vec![MintScan::new_noble()],
        Chain::Mantle => vec![MantleExplorer::new(), EtherScan::new(EVMChain::Mantle)],

        Chain::Ton => vec![TonViewer::new()],
        Chain::Tron => vec![TronScan::new()],
        Chain::Xrp => vec![XrpScan::new()],
        Chain::Aptos => vec![AptosExplorer::new(), AptosScan::new()],
        Chain::Sui => vec![SuiScan::new(), SuiVision::new()],
        Chain::Near => vec![NearBlocks::new()],
    }
}
