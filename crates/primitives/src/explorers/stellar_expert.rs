use crate::block_explorer::{BlockExplorer, Metadata};

static STELLAR_EXPERT_NAME: &str = "StellarExpert";
static STELLAR_EXPERT_BASE_URL: &str = "https://stellar.expert/explorer/public";

pub struct StellarExpert {
    pub meta: Metadata,
}

impl StellarExpert {
    pub fn new() -> Box<Self> {
        Box::new(Self {
            meta: Metadata {
                name: STELLAR_EXPERT_NAME,
                base_url: STELLAR_EXPERT_BASE_URL,
            },
        })
    }
}

impl BlockExplorer for StellarExpert {
    fn name(&self) -> String {
        self.meta.name.into()
    }

    fn get_tx_url(&self, hash: &str) -> String {
        format!("{}/tx/{}", self.meta.base_url, hash)
    }

    fn get_address_url(&self, address: &str) -> String {
        format!("{}/account/{}", self.meta.base_url, address)
    }

    fn get_token_url(&self, token: &str) -> Option<String> {
        Some(format!("{}/asset/{}", self.meta.base_url, token))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stellar_expert_account_url() {
        let explorer = StellarExpert::new();
        let address = "GBRCDFSBJFC4K3ZJV7WQVGDCROU6MCF3FDR3XGAMBG3ENKTGLNZHFKGJ";
        let url = explorer.get_address_url(address);
        assert_eq!(
            url,
            "https://stellar.expert/explorer/public/account/GBRCDFSBJFC4K3ZJV7WQVGDCROU6MCF3FDR3XGAMBG3ENKTGLNZHFKGJ"
        );
    }

    #[test]
    fn test_stellar_expert_token_url() {
        let explorer = StellarExpert::new();
        let token = "USDC-GA5ZSEJYB37JRC5AVCIA5MOP4RHTM335X2KGX3IHOJAPP5RE34K4KZVN";
        let url = explorer.get_token_url(token).unwrap();
        assert_eq!(
            url,
            "https://stellar.expert/explorer/public/asset/USDC-GA5ZSEJYB37JRC5AVCIA5MOP4RHTM335X2KGX3IHOJAPP5RE34K4KZVN"
        );
    }

    #[test]
    fn test_stellar_expert_tx_url() {
        let explorer = StellarExpert::new();
        let tx_hash = "249f71606fce57f3325e7abd8df7c8815594fe0fa986a2b6a838921190347bf6";
        let url = explorer.get_tx_url(tx_hash);
        assert_eq!(
            url,
            "https://stellar.expert/explorer/public/tx/249f71606fce57f3325e7abd8df7c8815594fe0fa986a2b6a838921190347bf6"
        );
    }
}
