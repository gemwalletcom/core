use crate::chain::Chain;
use crate::chain_evm::EVMChain;
use crate::explorers::{
    AlgorandAllo, AptosExplorer, AptosScan, BlockScout, Blockchair, Blocksec, Cardanocan, EtherScan, HyperLiquid, MantleExplorer, Mempool, MintScan,
    NearBlocks, OkxExplorer, RouteScan, RuneScan, ScopeExplorer, SolanaFM, Solscan, SubScan, SuiScan, SuiVision, TonScan, TonViewer, TronScan, Viewblock,
    XrpScan, ZkSync,
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
        Chain::BitcoinCash => vec![Blockchair::new_bitcoin_cash()],
        Chain::Litecoin => vec![Blockchair::new_litecoin()],
        Chain::Doge => vec![Blockchair::new_doge()],

        Chain::Ethereum => vec![
            EtherScan::new(EVMChain::Ethereum),
            Blockchair::new_ethereum(),
            Blocksec::new_ethereum(),
            ScopeExplorer::new(Chain::Ethereum),
        ],
        Chain::SmartChain => vec![
            EtherScan::new(EVMChain::SmartChain),
            Blockchair::new_bnb(),
            Blocksec::new_bsc(),
            ScopeExplorer::new(Chain::SmartChain),
        ],
        Chain::Polygon => vec![
            EtherScan::new(EVMChain::Polygon),
            Blockchair::new_polygon(),
            Blocksec::new_polygon(),
            ScopeExplorer::new(Chain::Polygon),
        ],
        Chain::Arbitrum => vec![
            EtherScan::new(EVMChain::Arbitrum),
            Blockchair::new_arbitrum(),
            Blocksec::new_arbitrum(),
            ScopeExplorer::new(Chain::Arbitrum),
        ],
        Chain::Optimism => vec![
            EtherScan::new(EVMChain::Optimism),
            Blockchair::new_optimism(),
            Blocksec::new_optimism(),
            ScopeExplorer::new(Chain::Optimism),
        ],
        Chain::Base => vec![
            EtherScan::new(EVMChain::Base),
            Blockchair::new_base(),
            Blocksec::new_base(),
            ScopeExplorer::new(Chain::Base),
        ],
        Chain::AvalancheC => vec![
            EtherScan::new(EVMChain::AvalancheC),
            RouteScan::new_avax(),
            Blockchair::new_avalanche(),
            ScopeExplorer::new(Chain::AvalancheC),
        ],
        Chain::OpBNB => vec![EtherScan::new(EVMChain::OpBNB), Blockchair::new_opbnb()],
        Chain::Fantom => vec![EtherScan::new(EVMChain::Fantom), Blockchair::new_fantom()],
        Chain::Gnosis => vec![EtherScan::new(EVMChain::Gnosis), Blockchair::new_gnosis()],
        Chain::Manta => vec![BlockScout::new_manta(), EtherScan::new(EVMChain::Manta)],
        Chain::Blast => vec![EtherScan::new(EVMChain::Blast)],
        Chain::Linea => vec![EtherScan::new(EVMChain::Linea), Blockchair::new_linea(), ScopeExplorer::new(Chain::Linea)],
        Chain::Celo => vec![BlockScout::new_celo(), EtherScan::new(EVMChain::Celo)],
        Chain::ZkSync => vec![ZkSync::new(), EtherScan::new(EVMChain::ZkSync)],
        Chain::World => vec![EtherScan::new(EVMChain::World)],
        Chain::Solana => vec![Solscan::new(), SolanaFM::new(), Blockchair::new_solana()],
        Chain::Thorchain => vec![RuneScan::new(), Viewblock::new()],

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
        Chain::Aptos => vec![AptosScan::new(), AptosExplorer::new(), Blockchair::new_aptos()],
        Chain::Sui => vec![SuiScan::new(), SuiVision::new()],
        Chain::Near => vec![NearBlocks::new()],
        Chain::Stellar => vec![Blockchair::new_stellar()],
        Chain::Sonic => vec![EtherScan::new(EVMChain::Sonic), RouteScan::new_sonic()],
        Chain::Algorand => vec![AlgorandAllo::new()],
        Chain::Polkadot => vec![SubScan::new_polkadot(), Blockchair::new_polkadot()],
        Chain::Cardano => vec![Cardanocan::new()],
        Chain::Abstract => vec![EtherScan::new(EVMChain::Abstract)],
        Chain::Berachain => vec![EtherScan::new(EVMChain::Berachain)],
        Chain::Ink => vec![RouteScan::new_ink(), BlockScout::new_ink(), OkxExplorer::new_ink()],
        Chain::Unichain => vec![EtherScan::new(EVMChain::Unichain)],
        Chain::Hyperliquid => vec![BlockScout::new_hyperliquid(), HyperLiquid::new()],
        Chain::Monad => vec![EtherScan::new(EVMChain::Monad)],
    }
}
