use primitives::{
    block_explorer::{get_block_explorer, BlockExplorer},
    chain::Chain,
    explorers::{MayanScan, RuneScan},
};
use std::str::FromStr;

use crate::swapper::SwapProvider;

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
        get_block_explorer(self.chain, explorer_name).get_tx_url(transaction_id)
    }

    pub fn get_transaction_swap_url(&self, explorer_name: &str, transaction_id: &str, provider: SwapProvider) -> String {
        match provider {
            SwapProvider::Mayan => MayanScan::new().get_tx_url(transaction_id),
            SwapProvider::Thorchain => RuneScan::new().get_tx_url(transaction_id),
            SwapProvider::UniswapV3
            | SwapProvider::UniswapV4
            | SwapProvider::PancakeSwapV3
            | SwapProvider::PancakeSwapAptosV2
            | SwapProvider::Orca
            | SwapProvider::Jupiter
            | SwapProvider::Across
            | SwapProvider::OkuTrade
            | SwapProvider::Wagmi
            | SwapProvider::Cetus
            | SwapProvider::StonFiV2
            | SwapProvider::Reservoir => self.get_transaction_url(explorer_name, transaction_id),
        }
    }

    pub fn get_address_url(&self, explorer_name: &str, address: &str) -> String {
        get_block_explorer(self.chain, explorer_name).get_address_url(address)
    }

    pub fn get_token_url(&self, explorer_name: &str, address: &str) -> Option<String> {
        get_block_explorer(self.chain, explorer_name).get_token_url(address)
    }

    pub fn get_validator_url(&self, explorer_name: &str, address: &str) -> Option<String> {
        get_block_explorer(self.chain, explorer_name).get_validator_url(address)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::block_explorer::get_block_explorers;

    #[test]
    fn test_bitcoin_explorers() {
        let chain = Chain::Bitcoin;
        let explorers = get_block_explorers(chain);

        assert_eq!(explorers.len(), 2);
        assert_eq!(explorers[0].name(), "Blockchair");
        assert_eq!(explorers[1].name(), "Mempool");

        let explorer = Explorer::new(chain.as_ref());
        let tx_url = explorer.get_transaction_url(&explorers[0].name(), "813d80363c09b1c4d3f0c6ce3382a048b320edefb573a8aedbc7ddd4c65cf7e4");

        assert_eq!(
            tx_url,
            "https://blockchair.com/bitcoin/transaction/813d80363c09b1c4d3f0c6ce3382a048b320edefb573a8aedbc7ddd4c65cf7e4"
        );

        let tx_url = explorer.get_transaction_url(&explorers[1].name(), "813d80363c09b1c4d3f0c6ce3382a048b320edefb573a8aedbc7ddd4c65cf7e4");

        assert_eq!(
            tx_url,
            "https://mempool.space/tx/813d80363c09b1c4d3f0c6ce3382a048b320edefb573a8aedbc7ddd4c65cf7e4"
        );
    }

    #[test]
    fn test_ethereum_explorers() {
        let chain = Chain::Ethereum;
        let explorers = get_block_explorers(chain);

        assert_eq!(explorers.len(), 3);
        assert_eq!(explorers[0].name(), "Etherscan");
        assert_eq!(explorers[1].name(), "Blockchair");

        let explorer = Explorer::new(chain.as_ref());
        let account_url = explorer.get_address_url(&explorers[0].name(), "0x1f9090aae28b8a3dceadf281b0f12828e676c326");
        let tx_url = explorer.get_transaction_url(&explorers[0].name(), "0xfd96a9ee20a7440bf65a5b8ecf7f884289ed78e28f82d45343a70f459e7a42a0");
        let token_url = explorer.get_token_url(&explorers[0].name(), "0xdac17f958d2ee523a2206206994597c13d831ec7");

        assert_eq!(account_url, "https://etherscan.io/address/0x1f9090aae28b8a3dceadf281b0f12828e676c326");
        assert_eq!(
            tx_url,
            "https://etherscan.io/tx/0xfd96a9ee20a7440bf65a5b8ecf7f884289ed78e28f82d45343a70f459e7a42a0"
        );
        assert_eq!(
            token_url,
            Some("https://etherscan.io/token/0xdac17f958d2ee523a2206206994597c13d831ec7".to_string())
        );
    }

    #[test]
    fn test_ton_explorer() {
        let chain = Chain::Ton;
        let explorers = get_block_explorers(chain);

        assert_eq!(explorers.len(), 3);
        assert_eq!(explorers[0].name(), "TonViewer");
        assert_eq!(explorers[1].name(), "Tonscan");
        assert_eq!(explorers[2].name(), "Blockchair");

        let explorer = Explorer::new(chain.as_ref());
        let account_url = explorer.get_address_url(&explorers[0].name(), "EQCxE6mUtQJKFnGfaROTKOt1lZbDiiX1kCixRv7Nw2Id_sDs");
        let token_url = explorer
            .get_token_url(&explorers[0].name(), "EQCxE6mUtQJKFnGfaROTKOt1lZbDiiX1kCixRv7Nw2Id_sDs")
            .unwrap();

        assert_eq!(account_url, "https://tonviewer.com/EQCxE6mUtQJKFnGfaROTKOt1lZbDiiX1kCixRv7Nw2Id_sDs");
        assert_eq!(token_url, account_url);

        let tx_url = explorer.get_transaction_url(&explorers[0].name(), "cefe5c6d145976c434280648fae28dfdfee58002e8c4e36195550ed6cdb22aa0");

        assert_eq!(
            tx_url,
            "https://tonviewer.com/transaction/cefe5c6d145976c434280648fae28dfdfee58002e8c4e36195550ed6cdb22aa0"
        );
    }

    #[test]
    fn test_solana_explorer() {
        let chain = Chain::Solana;
        let explorers = get_block_explorers(chain);

        assert_eq!(explorers.len(), 3);
        assert_eq!(explorers[1].name(), "SolanaFM");
        assert_eq!(explorers[0].name(), "Solscan");

        let explorer = Explorer::new(chain.as_ref());

        assert_eq!(
            explorer.get_address_url(&explorers[1].name(), "5x38Kp4hvdomTCnCrAny4UtMUt5rQBdB6px2K1Ui45Wq",),
            "https://solana.fm/address/5x38Kp4hvdomTCnCrAny4UtMUt5rQBdB6px2K1Ui45Wq"
        );
        assert_eq!(
            explorer.get_transaction_url(
                &explorers[1].name(),
                "58UdzFXAz6Vk58jEM6UsWmNb7kcJ1YvR2nQmkp8YQSW2gabmGra1u67SEjNZzTHCyuAn8NqzcQcn6qBLKx7uhVK7",
            ),
            "https://solana.fm/tx/58UdzFXAz6Vk58jEM6UsWmNb7kcJ1YvR2nQmkp8YQSW2gabmGra1u67SEjNZzTHCyuAn8NqzcQcn6qBLKx7uhVK7"
        );
        assert_eq!(
            explorer.get_transaction_url(
                &explorers[0].name(),
                "58UdzFXAz6Vk58jEM6UsWmNb7kcJ1YvR2nQmkp8YQSW2gabmGra1u67SEjNZzTHCyuAn8NqzcQcn6qBLKx7uhVK7",
            ),
            "https://solscan.io/tx/58UdzFXAz6Vk58jEM6UsWmNb7kcJ1YvR2nQmkp8YQSW2gabmGra1u67SEjNZzTHCyuAn8NqzcQcn6qBLKx7uhVK7"
        );
        assert_eq!(
            explorer
                .get_token_url(&explorers[1].name(), "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB",)
                .unwrap(),
            "https://solana.fm/address/Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB"
        );
    }

    #[test]
    fn test_cosmos_explorers() {
        let chain = Chain::Cosmos;
        let explorers = get_block_explorers(chain);

        assert_eq!(explorers.len(), 1);
        assert_eq!(explorers[0].name(), "Mintscan");

        let explorer = Explorer::new(chain.as_ref());
        let account_url = explorer.get_address_url(&explorers[0].name(), "cosmos1fxygpgus4nd5jmfl5j7fh5y8hyy53z8u95dzx7");
        let tx_url = explorer.get_transaction_url(&explorers[0].name(), "CFB4B38D75DB9D9055A7D4A2A76C67B8A27C37124C4E5663BEE104589E726763");
        let asset_url = explorer
            .get_token_url(&explorers[0].name(), "ibc/0025F8A87464A471E66B234C4F93AEC5B4DA3D42D7986451A059273426290DD5")
            .unwrap();

        assert_eq!(
            account_url,
            "https://www.mintscan.io/cosmos/address/cosmos1fxygpgus4nd5jmfl5j7fh5y8hyy53z8u95dzx7"
        );
        assert_eq!(
            tx_url,
            "https://www.mintscan.io/cosmos/tx/CFB4B38D75DB9D9055A7D4A2A76C67B8A27C37124C4E5663BEE104589E726763"
        );
        assert_eq!(
            asset_url,
            "https://www.mintscan.io/cosmos/assets/ibc/0025F8A87464A471E66B234C4F93AEC5B4DA3D42D7986451A059273426290DD5"
        )
    }

    #[test]
    fn test_noble_explorer() {
        let chain = Chain::Noble;
        let explorers = get_block_explorers(chain);

        assert_eq!(explorers.len(), 1);
        assert_eq!(explorers[0].name(), "Mintscan");

        let explorer = Explorer::new(chain.as_ref());
        let account_url = explorer.get_address_url(&explorers[0].name(), "noble17w8y9eujrz4m08nn0h349s5h2rs8uz5hqe02z4");
        let tx_url = explorer.get_transaction_url(&explorers[0].name(), "22F0B4F48A85925A668D64134B7377476DC5BAE3CF7CC38AFC0E17E5F7D90001");

        assert_eq!(
            account_url,
            "https://www.mintscan.io/noble/address/noble17w8y9eujrz4m08nn0h349s5h2rs8uz5hqe02z4"
        );
        assert_eq!(
            tx_url,
            "https://www.mintscan.io/noble/tx/22F0B4F48A85925A668D64134B7377476DC5BAE3CF7CC38AFC0E17E5F7D90001"
        );
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
            explorer.get_address_url(&explorers[0].name(), "0x6f02af629f66a13c5b8cb857cddf43804422d205b0bb9bda9db98b2635fe59bb",),
            "https://suiscan.xyz/mainnet/account/0x6f02af629f66a13c5b8cb857cddf43804422d205b0bb9bda9db98b2635fe59bb"
        );
        assert_eq!(
            explorer.get_transaction_url(&explorers[0].name(), "ArS7DzeHUA54ccRG12SqEZwt7snQePcanZ77Mkm2KRos",),
            "https://suiscan.xyz/mainnet/tx/ArS7DzeHUA54ccRG12SqEZwt7snQePcanZ77Mkm2KRos"
        );
        assert_eq!(
            explorer
                .get_validator_url(&explorers[0].name(), "0x61953ea72709eed72f4441dd944eec49a11b4acabfc8e04015e89c63be81b6ab",)
                .unwrap(),
            "https://suiscan.xyz/mainnet/validator/0x61953ea72709eed72f4441dd944eec49a11b4acabfc8e04015e89c63be81b6ab"
        );

        assert_eq!(
            explorer.get_address_url(&explorers[1].name(), "0x6f02af629f66a13c5b8cb857cddf43804422d205b0bb9bda9db98b2635fe59bb",),
            "https://suivision.xyz/account/0x6f02af629f66a13c5b8cb857cddf43804422d205b0bb9bda9db98b2635fe59bb"
        );
        assert_eq!(
            explorer.get_transaction_url(&explorers[1].name(), "ArS7DzeHUA54ccRG12SqEZwt7snQePcanZ77Mkm2KRos",),
            "https://suivision.xyz/txblock/ArS7DzeHUA54ccRG12SqEZwt7snQePcanZ77Mkm2KRos"
        );
    }

    #[test]
    fn test_tronscan() {
        let chain = Chain::Tron;
        let explorers = get_block_explorers(chain);

        assert_eq!(explorers.len(), 2);
        assert_eq!(explorers[0].name(), "TRONSCAN");

        let explorer = Explorer::new(chain.as_ref());
        let account_url = explorer.get_address_url(&explorers[0].name(), "TJApZYJwPKuQR7tL6FmvD6jDjbYpHESZGH");
        let tx_url = explorer.get_transaction_url(&explorers[0].name(), "4e55fe0a528240152ab566dc11ce593a30c1d2cfd0fc91f0c555887639eab2db");

        assert_eq!(account_url, "https://tronscan.org/#/address/TJApZYJwPKuQR7tL6FmvD6jDjbYpHESZGH");
        assert_eq!(
            tx_url,
            "https://tronscan.org/#/transaction/4e55fe0a528240152ab566dc11ce593a30c1d2cfd0fc91f0c555887639eab2db"
        );
    }

    #[test]
    fn test_runescan() {
        let explorers = get_block_explorers(Chain::Thorchain);

        assert_eq!(explorers.len(), 2);
        assert_eq!(explorers[0].name(), "RuneScan");

        let explorer = Explorer::new(Chain::Thorchain.as_ref());
        let account_url: String = explorer.get_address_url(&explorers[0].name(), "thor166n4w5039meulfa3p6ydg60ve6ueac7tlt0jws");
        let tx_url = explorer.get_transaction_url(&explorers[0].name(), "FF82C517ECFDCA71A6CD3501063D76995C67509B2AFC012D2BCE61C130C05E98");

        assert_eq!(account_url, "https://runescan.io/address/thor166n4w5039meulfa3p6ydg60ve6ueac7tlt0jws");
        assert_eq!(
            tx_url,
            "https://runescan.io/tx/FF82C517ECFDCA71A6CD3501063D76995C67509B2AFC012D2BCE61C130C05E98"
        );
    }

    #[test]
    fn test_transaction_swap_url() {
        let explorers = get_block_explorers(Chain::Thorchain);
        let explorer = Explorer::new(Chain::Thorchain.as_ref());
        let tx_url = explorer.get_transaction_swap_url(
            &explorers[0].name(),
            "0x0299923c9a0a40e3a296058ac2c5c3a7b41f91803ea36ad9645492ccca0f8631",
            SwapProvider::Thorchain,
        );

        assert_eq!(
            tx_url,
            "https://runescan.io/tx/0299923c9a0a40e3a296058ac2c5c3a7b41f91803ea36ad9645492ccca0f8631"
        );

        let tx_url = explorer.get_transaction_swap_url(
            &explorers[0].name(),
            "0x56acc6a58fc0bdd9e9be5cc2a3ff079b91b933f562cf0fe760f1d8d6b76f4876",
            SwapProvider::Mayan,
        );

        assert_eq!(
            tx_url,
            "https://explorer.mayan.finance/tx/0x56acc6a58fc0bdd9e9be5cc2a3ff079b91b933f562cf0fe760f1d8d6b76f4876"
        );
    }
}
