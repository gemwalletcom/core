use crate::block_explorer::BlockExplorer;
use crate::explorers::metadata::{Explorer, Metadata, TX_PATH, ACCOUNT_PATH, ASSET_PATH};

pub fn new() -> Box<dyn BlockExplorer> {
    Explorer::boxed(Metadata {
        name: "StellarExpert",
        base_url: "https://stellar.expert/explorer/public",
        tx_path: TX_PATH,
        address_path: ACCOUNT_PATH,
        token_path: Some(ASSET_PATH),
        validator_path: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stellar_expert_account_url() {
        let explorer = new();
        let address = "GBRCDFSBJFC4K3ZJV7WQVGDCROU6MCF3FDR3XGAMBG3ENKTGLNZHFKGJ";
        let url = explorer.get_address_url(address);
        assert_eq!(
            url,
            "https://stellar.expert/explorer/public/account/GBRCDFSBJFC4K3ZJV7WQVGDCROU6MCF3FDR3XGAMBG3ENKTGLNZHFKGJ"
        );
    }

    #[test]
    fn test_stellar_expert_token_url() {
        let explorer = new();
        let token = "USDC-GA5ZSEJYB37JRC5AVCIA5MOP4RHTM335X2KGX3IHOJAPP5RE34K4KZVN";
        let url = explorer.get_token_url(token).unwrap();
        assert_eq!(
            url,
            "https://stellar.expert/explorer/public/asset/USDC-GA5ZSEJYB37JRC5AVCIA5MOP4RHTM335X2KGX3IHOJAPP5RE34K4KZVN"
        );
    }

    #[test]
    fn test_stellar_expert_tx_url() {
        let explorer = new();
        let tx_hash = "249f71606fce57f3325e7abd8df7c8815594fe0fa986a2b6a838921190347bf6";
        let url = explorer.get_tx_url(tx_hash);
        assert_eq!(
            url,
            "https://stellar.expert/explorer/public/tx/249f71606fce57f3325e7abd8df7c8815594fe0fa986a2b6a838921190347bf6"
        );
    }
}