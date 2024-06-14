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

    pub fn get_transaction_url(&self, explorer_name: &str, transaction_id: &str) -> String {
        get_block_explorers(self.chain)
            .into_iter()
            .find(|x| x.name() == explorer_name)
            .unwrap()
            .get_tx_url(transaction_id)
    }

    pub fn get_address_url(&self, explorer_name: &str, address: &str) -> String {
        get_block_explorers(self.chain)
            .into_iter()
            .find(|x| x.name() == explorer_name)
            .unwrap()
            .get_address_url(address)
    }

    pub fn get_token_url(&self, explorer_name: &str, address: &str) -> Option<String> {
        get_block_explorers(self.chain)
            .into_iter()
            .find(|x| x.name() == explorer_name)
            .unwrap()
            .get_token_url(address)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bitcoin_explorers() {
        let chain = Chain::Bitcoin;
        let explorers = get_block_explorers(chain);

        assert_eq!(explorers.len(), 2);
        assert_eq!(explorers[0].name(), "Blockchair");
        assert_eq!(explorers[1].name(), "Mempool");

        let explorer = Explorer::new(chain.as_ref());
        let tx_url = explorer.get_transaction_url(
            &explorers[0].name(),
            "813d80363c09b1c4d3f0c6ce3382a048b320edefb573a8aedbc7ddd4c65cf7e4",
        );

        assert_eq!(tx_url, "https://blockchair.com/bitcoin/transaction/813d80363c09b1c4d3f0c6ce3382a048b320edefb573a8aedbc7ddd4c65cf7e4");

        let tx_url = explorer.get_transaction_url(
            &explorers[1].name(),
            "813d80363c09b1c4d3f0c6ce3382a048b320edefb573a8aedbc7ddd4c65cf7e4",
        );

        assert_eq!(tx_url, "https://mempool.space/tx/813d80363c09b1c4d3f0c6ce3382a048b320edefb573a8aedbc7ddd4c65cf7e4");
    }

    #[test]
    fn test_ethereum_explorers() {
        let chain = Chain::Ethereum;
        let explorers = get_block_explorers(chain);

        assert_eq!(explorers.len(), 1);
        assert_eq!(explorers[0].name(), "Etherscan");

        let explorer = Explorer::new(chain.as_ref());
        let account_url = explorer.get_address_url(
            &explorers[0].name(),
            "0x1f9090aae28b8a3dceadf281b0f12828e676c326",
        );
        let tx_url = explorer.get_transaction_url(
            &explorers[0].name(),
            "0xfd96a9ee20a7440bf65a5b8ecf7f884289ed78e28f82d45343a70f459e7a42a0",
        );
        let token_url = explorer.get_token_url(
            &explorers[0].name(),
            "0xdac17f958d2ee523a2206206994597c13d831ec7",
        );

        assert_eq!(
            account_url,
            "https://etherscan.io/address/0x1f9090aae28b8a3dceadf281b0f12828e676c326"
        );
        assert_eq!(tx_url, "https://etherscan.io/tx/0xfd96a9ee20a7440bf65a5b8ecf7f884289ed78e28f82d45343a70f459e7a42a0");
        assert_eq!(
            token_url,
            Some(
                "https://etherscan.io/token/0xdac17f958d2ee523a2206206994597c13d831ec7".to_string()
            )
        );
    }

    #[test]
    fn test_ton_explorer() {
        let chain = Chain::Ton;
        let explorers = get_block_explorers(chain);

        assert_eq!(explorers.len(), 1);
        assert_eq!(explorers[0].name(), "TonViewer");

        let explorer = Explorer::new(chain.as_ref());
        let account_url = explorer.get_address_url(
            &explorers[0].name(),
            "EQCxE6mUtQJKFnGfaROTKOt1lZbDiiX1kCixRv7Nw2Id_sDs",
        );
        let token_url = explorer
            .get_token_url(
                &explorers[0].name(),
                "EQCxE6mUtQJKFnGfaROTKOt1lZbDiiX1kCixRv7Nw2Id_sDs",
            )
            .unwrap();

        assert_eq!(
            account_url,
            "https://tonviewer.com/EQCxE6mUtQJKFnGfaROTKOt1lZbDiiX1kCixRv7Nw2Id_sDs"
        );
        assert_eq!(token_url, account_url);

        let tx_url = explorer.get_transaction_url(
            &explorers[0].name(),
            "cefe5c6d145976c434280648fae28dfdfee58002e8c4e36195550ed6cdb22aa0",
        );

        assert_eq!(tx_url, "https://tonviewer.com/transaction/cefe5c6d145976c434280648fae28dfdfee58002e8c4e36195550ed6cdb22aa0");
    }

    #[test]
    fn test_solana_explorer() {
        let chain = Chain::Solana;
        let explorers = get_block_explorers(chain);

        assert_eq!(explorers.len(), 2);
        assert_eq!(explorers[0].name(), "SolanaFM");
        assert_eq!(explorers[1].name(), "Solscan");

        let explorer = Explorer::new(chain.as_ref());

        assert_eq!(
            explorer.get_address_url(
                &explorers[0].name(),
                "5x38Kp4hvdomTCnCrAny4UtMUt5rQBdB6px2K1Ui45Wq",
            ),
            "https://solana.fm/address/5x38Kp4hvdomTCnCrAny4UtMUt5rQBdB6px2K1Ui45Wq"
        );
        assert_eq!(
            explorer.get_transaction_url(
                &explorers[0].name(),
                "58UdzFXAz6Vk58jEM6UsWmNb7kcJ1YvR2nQmkp8YQSW2gabmGra1u67SEjNZzTHCyuAn8NqzcQcn6qBLKx7uhVK7",
            ),
            "https://solana.fm/tx/58UdzFXAz6Vk58jEM6UsWmNb7kcJ1YvR2nQmkp8YQSW2gabmGra1u67SEjNZzTHCyuAn8NqzcQcn6qBLKx7uhVK7"
        );
        assert_eq!(
            explorer.get_transaction_url(
                &explorers[1].name(),
                "58UdzFXAz6Vk58jEM6UsWmNb7kcJ1YvR2nQmkp8YQSW2gabmGra1u67SEjNZzTHCyuAn8NqzcQcn6qBLKx7uhVK7",
            ),
            "https://solscan.io/tx/58UdzFXAz6Vk58jEM6UsWmNb7kcJ1YvR2nQmkp8YQSW2gabmGra1u67SEjNZzTHCyuAn8NqzcQcn6qBLKx7uhVK7"
        );
        assert_eq!(
            explorer
                .get_token_url(
                    &explorers[0].name(),
                    "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB",
                )
                .unwrap(),
            "https://solana.fm/address/Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB"
        );
    }

    #[test]
    fn test_cosmos_explorers() {
        let chain = Chain::Cosmos;
        let explorers = get_block_explorers(chain);

        assert_eq!(explorers.len(), 1);
        assert_eq!(explorers[0].name(), "MintScan");

        let explorer = Explorer::new(chain.as_ref());
        let account_url = explorer.get_address_url(
            &explorers[0].name(),
            "cosmos1fxygpgus4nd5jmfl5j7fh5y8hyy53z8u95dzx7",
        );
        let tx_url = explorer.get_transaction_url(
            &explorers[0].name(),
            "CFB4B38D75DB9D9055A7D4A2A76C67B8A27C37124C4E5663BEE104589E726763",
        );

        assert_eq!(
            account_url,
            "https://mintscan.io/cosmos/account/cosmos1fxygpgus4nd5jmfl5j7fh5y8hyy53z8u95dzx7"
        );
        assert_eq!(
            tx_url,
            "https://mintscan.io/cosmos/tx/CFB4B38D75DB9D9055A7D4A2A76C67B8A27C37124C4E5663BEE104589E726763"
        )
    }

    #[test]
    fn test_sui_vision() {
        let chain = Chain::Sui;
        let explorers = get_block_explorers(chain);

        assert_eq!(explorers.len(), 2);
        assert_eq!(explorers[0].name(), "SuiScan");
        assert_eq!(explorers[1].name(), "SuiVision");

        let explorer = Explorer::new(chain.as_ref());

        assert_eq!(
            explorer.get_address_url(
                &explorers[0].name(),
                "0x6f02af629f66a13c5b8cb857cddf43804422d205b0bb9bda9db98b2635fe59bb",
            ),
            "https://suiscan.xyz/mainnet/account/0x6f02af629f66a13c5b8cb857cddf43804422d205b0bb9bda9db98b2635fe59bb"
        );
        assert_eq!(
            explorer.get_transaction_url(
                &explorers[0].name(),
                "ArS7DzeHUA54ccRG12SqEZwt7snQePcanZ77Mkm2KRos",
            ),
            "https://suiscan.xyz/mainnet/tx/ArS7DzeHUA54ccRG12SqEZwt7snQePcanZ77Mkm2KRos"
        );

        assert_eq!(
            explorer.get_address_url(
                &explorers[1].name(),
                "0x6f02af629f66a13c5b8cb857cddf43804422d205b0bb9bda9db98b2635fe59bb",
            ),
            "https://suivision.xyz/account/0x6f02af629f66a13c5b8cb857cddf43804422d205b0bb9bda9db98b2635fe59bb"
        );
        assert_eq!(
            explorer.get_transaction_url(
                &explorers[1].name(),
                "ArS7DzeHUA54ccRG12SqEZwt7snQePcanZ77Mkm2KRos",
            ),
            "https://suivision.xyz/txblock/ArS7DzeHUA54ccRG12SqEZwt7snQePcanZ77Mkm2KRos"
        );
    }

    #[test]
    fn test_tronscan() {
        let chain = Chain::Tron;
        let explorers = get_block_explorers(chain);

        assert_eq!(explorers.len(), 1);
        assert_eq!(explorers[0].name(), "TRONSCAN");

        let explorer = Explorer::new(chain.as_ref());
        let account_url =
            explorer.get_address_url(&explorers[0].name(), "TJApZYJwPKuQR7tL6FmvD6jDjbYpHESZGH");
        let tx_url = explorer.get_transaction_url(
            &explorers[0].name(),
            "4e55fe0a528240152ab566dc11ce593a30c1d2cfd0fc91f0c555887639eab2db",
        );

        assert_eq!(
            account_url,
            "https://tronscan.org/#/address/TJApZYJwPKuQR7tL6FmvD6jDjbYpHESZGH"
        );
        assert_eq!(
            tx_url,
            "https://tronscan.org/#/transaction/4e55fe0a528240152ab566dc11ce593a30c1d2cfd0fc91f0c555887639eab2db"
        );
    }
}
