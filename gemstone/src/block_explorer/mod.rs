mod explorers;
use explorers::{
    AptosExplorer, AptosScan, BlockScout, Blockchair, EtherScan, MantleExplorer, Mempool, MintScan,
    NearBlocks, SolanaFM, SuiScan, SuiVision, TonViewer, TronScan, Viewblock, XrpScan, ZkSync,
};
use primitives::Chain;
use std::str::FromStr;

#[uniffi::export]
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
        Chain::Bitcoin => vec![Box::new(Blockchair::new(chain)), Box::new(Mempool::new())],
        Chain::Litecoin => vec![Box::new(Blockchair::new(chain))],
        Chain::Doge => vec![Box::new(Blockchair::new(chain))],

        Chain::Ethereum => vec![Box::new(EtherScan::new_evm(chain))],
        Chain::SmartChain => vec![Box::new(EtherScan::new_evm(chain))],
        Chain::Polygon => vec![Box::new(EtherScan::new_evm(chain))],
        Chain::Arbitrum => vec![Box::new(EtherScan::new_evm(chain))],
        Chain::Optimism => vec![Box::new(EtherScan::new_evm(chain))],
        Chain::Base => vec![Box::new(EtherScan::new_evm(chain))],
        Chain::AvalancheC => vec![Box::new(EtherScan::new_evm(chain))],
        Chain::OpBNB => vec![Box::new(EtherScan::new_evm(chain))],
        Chain::Fantom => vec![Box::new(EtherScan::new_evm(chain))],
        Chain::Gnosis => vec![Box::new(EtherScan::new_evm(chain))],
        Chain::Manta => vec![Box::new(EtherScan::new_evm(chain))],
        Chain::Blast => vec![Box::new(EtherScan::new_evm(chain))],
        Chain::Linea => vec![Box::new(EtherScan::new_evm(chain))],
        Chain::Celo => vec![
            Box::new(BlockScout::new(chain)),
            Box::new(EtherScan::new_evm(chain)),
        ],
        Chain::ZkSync => vec![Box::new(ZkSync::new()), Box::new(EtherScan::new_evm(chain))],
        Chain::Solana => vec![Box::new(SolanaFM::new()), Box::new(EtherScan::solana())],
        Chain::Thorchain => vec![Box::new(Viewblock::new())],

        Chain::Cosmos => vec![Box::new(MintScan::new(chain))],
        Chain::Osmosis => vec![Box::new(MintScan::new(chain))],
        Chain::Celestia => vec![Box::new(MintScan::new(chain))],
        Chain::Injective => vec![Box::new(MintScan::new(chain))],
        Chain::Sei => vec![Box::new(MintScan::new(chain))],
        Chain::Mantle => vec![
            Box::new(MantleExplorer::new()),
            Box::new(EtherScan::new_evm(chain)),
        ],
        Chain::Noble => vec![Box::new(MintScan::new(chain))],

        Chain::Ton => vec![Box::new(TonViewer::new())],
        Chain::Tron => vec![Box::new(TronScan::new())],
        Chain::Xrp => vec![Box::new(XrpScan::new())],
        Chain::Aptos => vec![Box::new(AptosExplorer::new()), Box::new(AptosScan::new())],
        Chain::Sui => vec![Box::new(SuiScan::new()), Box::new(SuiVision::new())],
        Chain::Near => vec![Box::new(NearBlocks::new())],
    }
}

/// Explorer
#[derive(uniffi::Object)]
pub struct Explorer {
    pub chain: Chain,
}

#[uniffi::export]
impl Explorer {
    #[uniffi::constructor]
    fn new(chain: &str) -> Self {
        Self {
            chain: Chain::from_str(chain).unwrap(),
        }
    }

    pub fn get_transaction_url(&self, explorer_name: &str, transaction_id: String) -> String {
        get_block_explorers(self.chain)
            .into_iter()
            .find(|x| x.name() == explorer_name)
            .unwrap()
            .get_tx_url(&transaction_id)
    }

    pub fn get_address_url(&self, explorer_name: &str, address: String) -> String {
        get_block_explorers(self.chain)
            .into_iter()
            .find(|x| x.name() == explorer_name)
            .unwrap()
            .get_address_url(&address)
    }

    pub fn get_token_url(&self, explorer_name: &str, address: String) -> Option<String> {
        get_block_explorers(self.chain)
            .into_iter()
            .find(|x| x.name() == explorer_name)
            .unwrap()
            .get_token_url(&address)
    }
}
