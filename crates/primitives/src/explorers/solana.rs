use crate::block_explorer::BlockExplorer;
use crate::explorers::metadata::{ACCOUNT_PATH, ADDRESS_PATH, TOKEN_PATH, TX_PATH};

struct SolanaExplorer {
    name: &'static str,
    base_url: &'static str,
    address_path: &'static str,
    token_path: &'static str,
}

impl SolanaExplorer {
    fn boxed(
        name: &'static str,
        base_url: &'static str,
        address_path: &'static str,
        token_path: &'static str,
    ) -> Box<dyn BlockExplorer> {
        Box::new(Self {
            name,
            base_url,
            address_path,
            token_path,
        })
    }
}

impl BlockExplorer for SolanaExplorer {
    fn name(&self) -> String {
        self.name.into()
    }

    fn get_tx_url(&self, hash: &str) -> String {
        format!("{}{}/{}", self.base_url, TX_PATH, hash)
    }

    fn get_address_url(&self, address: &str) -> String {
        format!("{}{}/{}", self.base_url, self.address_path, address)
    }

    fn get_token_url(&self, token: &str) -> Option<String> {
        Some(format!("{}{}/{}", self.base_url, self.token_path, token))
    }

    fn get_nft_url(&self, _contract: &str, token_id: &str) -> Option<String> {
        self.get_token_url(token_id)
    }
}

pub fn new_solscan() -> Box<dyn BlockExplorer> {
    SolanaExplorer::boxed("Solscan", "https://solscan.io", ACCOUNT_PATH, TOKEN_PATH)
}

pub fn new_solana_fm() -> Box<dyn BlockExplorer> {
    SolanaExplorer::boxed("SolanaFM", "https://solana.fm", ADDRESS_PATH, ADDRESS_PATH)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_solscan_urls() {
        let explorer = new_solscan();

        assert_eq!(explorer.name(), "Solscan");
        assert_eq!(
            explorer.get_tx_url("ArS7DzeHUA54ccRG12SqEZwt7snQePcanZ77Mkm2KRos"),
            "https://solscan.io/tx/ArS7DzeHUA54ccRG12SqEZwt7snQePcanZ77Mkm2KRos"
        );
        assert_eq!(
            explorer.get_address_url("GvhwZwtV32kYUXUw965CUM3KGPdtBsDwPVpi92brY5R2"),
            "https://solscan.io/account/GvhwZwtV32kYUXUw965CUM3KGPdtBsDwPVpi92brY5R2"
        );
        assert_eq!(
            explorer.get_token_url("GvhwZwtV32kYUXUw965CUM3KGPdtBsDwPVpi92brY5R2"),
            Some("https://solscan.io/token/GvhwZwtV32kYUXUw965CUM3KGPdtBsDwPVpi92brY5R2".to_string())
        );
        assert_eq!(
            explorer.get_nft_url("ignored-contract", "GvhwZwtV32kYUXUw965CUM3KGPdtBsDwPVpi92brY5R2"),
            Some("https://solscan.io/token/GvhwZwtV32kYUXUw965CUM3KGPdtBsDwPVpi92brY5R2".to_string())
        );
    }

    #[test]
    fn test_solana_fm_urls() {
        let explorer = new_solana_fm();

        assert_eq!(explorer.name(), "SolanaFM");
        assert_eq!(
            explorer.get_tx_url("ArS7DzeHUA54ccRG12SqEZwt7snQePcanZ77Mkm2KRos"),
            "https://solana.fm/tx/ArS7DzeHUA54ccRG12SqEZwt7snQePcanZ77Mkm2KRos"
        );
        assert_eq!(
            explorer.get_address_url("GvhwZwtV32kYUXUw965CUM3KGPdtBsDwPVpi92brY5R2"),
            "https://solana.fm/address/GvhwZwtV32kYUXUw965CUM3KGPdtBsDwPVpi92brY5R2"
        );
        assert_eq!(
            explorer.get_token_url("GvhwZwtV32kYUXUw965CUM3KGPdtBsDwPVpi92brY5R2"),
            Some("https://solana.fm/address/GvhwZwtV32kYUXUw965CUM3KGPdtBsDwPVpi92brY5R2".to_string())
        );
        assert_eq!(
            explorer.get_nft_url("ignored-contract", "GvhwZwtV32kYUXUw965CUM3KGPdtBsDwPVpi92brY5R2"),
            Some("https://solana.fm/address/GvhwZwtV32kYUXUw965CUM3KGPdtBsDwPVpi92brY5R2".to_string())
        );
    }
}
