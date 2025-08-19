use crate::block_explorer::BlockExplorer;
use crate::chain_evm::EVMChain;
use crate::explorers::metadata::{Explorer, Metadata};

pub struct EtherScan;

impl EtherScan {
    pub fn boxed(chain: EVMChain) -> Box<dyn BlockExplorer> {
        match chain {
            EVMChain::Ethereum => Explorer::boxed(Metadata::with_token("Etherscan", "https://etherscan.io")),
            EVMChain::SmartChain => Explorer::boxed(Metadata::with_token("BscScan", "https://bscscan.com")),
            EVMChain::Polygon => Explorer::boxed(Metadata::with_token("PolygonScan", "https://polygonscan.com")),
            EVMChain::Arbitrum => Explorer::boxed(Metadata::with_token("ArbiScan", "https://arbiscan.io")),
            EVMChain::Optimism => Explorer::boxed(Metadata::with_token("Etherscan", "https://optimistic.etherscan.io")),
            EVMChain::Base => Explorer::boxed(Metadata::with_token("BaseScan", "https://basescan.org")),
            EVMChain::AvalancheC => Explorer::boxed(Metadata::with_token("SnowScan", "https://snowscan.xyz")),
            EVMChain::OpBNB => Explorer::boxed(Metadata::with_token("opBNBScan", "https://opbnb.bscscan.com")),
            EVMChain::Fantom => Explorer::boxed(Metadata::with_token("FTMScan", "https://ftmscan.com")),
            EVMChain::Gnosis => Explorer::boxed(Metadata::with_token("GnosisScan", "https://gnosisscan.io")),
            EVMChain::Manta => Explorer::boxed(Metadata::with_token("Socialscan", "https://manta.socialscan.io")),
            EVMChain::Blast => Explorer::boxed(Metadata::with_token("BlastScan", "https://blastscan.io")),
            EVMChain::Linea => Explorer::boxed(Metadata::with_token("LineaScan", "https://lineascan.build")),
            EVMChain::ZkSync => Explorer::boxed(Metadata::with_token("zkSync Era Explorer", "https://era.zksync.network")),
            EVMChain::Celo => Explorer::boxed(Metadata::with_token("CeloScan", "https://celoscan.io")),
            EVMChain::Mantle => Explorer::boxed(Metadata::with_token("MantleScan", "https://mantlescan.xyz")),
            EVMChain::World => Explorer::boxed(Metadata::with_token("WorldScan", "https://worldscan.org")),
            EVMChain::Sonic => Explorer::boxed(Metadata::with_token("SonicScan", "https://sonicscan.org")),
            EVMChain::Abstract => Explorer::boxed(Metadata::with_token("Abscan", "https://abscan.org")),
            EVMChain::Berachain => Explorer::boxed(Metadata::with_token("Berascan", "https://berascan.com")),
            EVMChain::Unichain => Explorer::boxed(Metadata::with_token("Uniscan", "https://uniscan.xyz")),
            EVMChain::Monad => Explorer::boxed(Metadata::with_token("Monad", "https://testnet.monadexplorer.com")),
            EVMChain::Hyperliquid => Explorer::boxed(Metadata::with_token("HyperEvmScan", "https://hyperevmscan.io")),
            _ => todo!(),
        }
    }
}
