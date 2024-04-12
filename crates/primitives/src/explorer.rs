use crate::Chain;

pub struct Explorer<'a> {
    #[allow(dead_code)]
    pub host: &'a str,
    pub transaction_url: &'a str,
    pub account_url: &'a str,
    pub token_url: Option<&'a str>,
}

impl<'a> Explorer<'a> {
    pub fn get_explorer_url(chain: Chain) -> Explorer<'static> {
        match chain {
            Chain::Bitcoin => Explorer {
                host: "https://blockchair.com",
                transaction_url: "/bitcoin/transaction/",
                account_url: "/bitcoin/address/",
                token_url: None,
            },
            Chain::Litecoin => Explorer {
                host: "https://blockchair.com",
                transaction_url: "/litecoin/transaction/",
                account_url: "/litecoin/address/",
                token_url: None,
            },
            Chain::Ethereum => Explorer {
                host: "https://etherscan.io",
                transaction_url: "/tx/",
                account_url: "/address/",
                token_url: Some("/token/"),
            },
            Chain::SmartChain => Explorer {
                host: "https://bscscan.com",
                transaction_url: "/tx/",
                account_url: "/address/",
                token_url: Some("/token/"),
            },
            Chain::Solana => Explorer {
                host: "https://solana.fm",
                transaction_url: "/tx/",
                account_url: "/address/",
                token_url: Some("/address/"),
            },
            Chain::Polygon => Explorer {
                host: "https://polygonscan.com",
                transaction_url: "/tx/",
                account_url: "/address/",
                token_url: Some("/token/"),
            },
            Chain::Thorchain => Explorer {
                host: "https://viewblock.io/thorchain",
                transaction_url: "/tx/",
                account_url: "/address/",
                token_url: None,
            },
            Chain::Cosmos => Explorer {
                host: "https://mintscan.io/cosmos",
                transaction_url: "/tx/",
                account_url: "/address/",
                token_url: None,
            },
            Chain::Osmosis => Explorer {
                host: "https://mintscan.io/osmosis",
                transaction_url: "/tx/",
                account_url: "/address/",
                token_url: None,
            },
            Chain::Arbitrum => Explorer {
                host: "https://arbiscan.io",
                transaction_url: "/tx/",
                account_url: "/address/",
                token_url: Some("/token/"),
            },
            Chain::Ton => Explorer {
                host: "https://tonscan.org",
                transaction_url: "/tx/",
                account_url: "/address/",
                token_url: None,
            },
            Chain::Tron => Explorer {
                host: "https://tronscan.org",
                transaction_url: "/#/transaction/",
                account_url: "/#/address/",
                token_url: Some("/#/token20/"),
            },
            Chain::Doge => Explorer {
                host: "https://blockchair.com",
                transaction_url: "/dogecoin/transaction/",
                account_url: "/dogecoin/address/",
                token_url: None,
            },
            Chain::Optimism => Explorer {
                host: "https://optimistic.etherscan.io",
                transaction_url: "/tx/",
                account_url: "/address/",
                token_url: Some("/token/"),
            },
            Chain::Aptos => Explorer {
                host: "https://explorer.aptoslabs.com",
                transaction_url: "/txn/",
                account_url: "/account/",
                token_url: None,
            },
            Chain::Base => Explorer {
                host: "https://basescan.org",
                transaction_url: "/tx/",
                account_url: "/address/",
                token_url: Some("/token/"),
            },
            Chain::AvalancheC => Explorer {
                host: "https://snowtrace.io",
                transaction_url: "/tx/",
                account_url: "/address/",
                token_url: Some("/token/"),
            },
            Chain::Sui => Explorer {
                host: "https://suiscan.xyz",
                transaction_url: "/mainnet/tx/",
                account_url: "/mainnet/account/",
                token_url: Some("/mainnet/account/"),
            },
            Chain::Xrp => Explorer {
                host: "https://xrpscan.com",
                transaction_url: "/tx/",
                account_url: "/account/",
                token_url: None,
            },
            Chain::OpBNB => Explorer {
                host: "https://opbnb.bscscan.com",
                transaction_url: "/tx/",
                account_url: "/address/",
                token_url: Some("/token/"),
            },
            Chain::Fantom => Explorer {
                host: "https://ftmscan.com",
                transaction_url: "/tx/",
                account_url: "/address/",
                token_url: Some("/token/"),
            },
            Chain::Gnosis => Explorer {
                host: "https://gnosisscan.io",
                transaction_url: "/tx/",
                account_url: "/address/",
                token_url: Some("/token/"),
            },
            Chain::Celestia => Explorer {
                host: "https://mintscan.io/celestia",
                transaction_url: "/tx/",
                account_url: "/address/",
                token_url: None,
            },
            Chain::Injective => Explorer {
                host: "https://mintscan.io/sei",
                transaction_url: "/tx/",
                account_url: "/address/",
                token_url: None,
            },
            Chain::Sei => Explorer {
                host: "https://mintscan.io/sei",
                transaction_url: "/txs/",
                account_url: "/address/",
                token_url: None,
            },
            Chain::Manta => Explorer {
                host: "https://pacific-explorer.manta.network",
                transaction_url: "/tx/",
                account_url: "/address/",
                token_url: Some("/token/"),
            },
            Chain::Blast => Explorer {
                host: "https://blastscan.io",
                transaction_url: "/tx/",
                account_url: "/address/",
                token_url: Some("/token/"),
            },
            Chain::Noble => Explorer {
                host: "https://mintscan.io/noble",
                transaction_url: "/tx/",
                account_url: "/address/",
                token_url: None,
            },
            Chain::ZkSync => Explorer {
                host: "https://explorer.zksync.io",
                transaction_url: "/tx/",
                account_url: "/address/",
                token_url: None,
            },
            Chain::Linea => Explorer {
                host: "https://lineascan.build",
                transaction_url: "/tx/",
                account_url: "/address/",
                token_url: None,
            },
            Chain::Mantle => Explorer {
                host: "https://explorer.mantle.xyz",
                transaction_url: "/tx/",
                account_url: "/address/",
                token_url: None,
            },
            Chain::Celo => Explorer {
                host: "https://explorer.celo.org/mainnet",
                transaction_url: "/tx/",
                account_url: "/address/",
                token_url: None,
            },
            Chain::Near => Explorer {
                host: "https://nearblocks.io",
                transaction_url: "/txns/",
                account_url: "/address/",
                token_url: None,
            },
            Chain::Dymension => Explorer {
                host: "https://www.mintscan.io/dymension",
                transaction_url: "/tx/",
                account_url: "/address/",
                token_url: Some("/assets/"),
            },
            Chain::Saga => Explorer {
                host: "https://www.mintscan.io/saga",
                transaction_url: "/tx/",
                account_url: "/address/",
                token_url: None,
            },
        }
    }

    pub fn get_explorer_transaction_url(chain: Chain, transaction_id: &str) -> String {
        let explorer = Explorer::get_explorer_url(chain);
        format!(
            "{}{}{}",
            explorer.host, explorer.transaction_url, transaction_id
        )
    }

    pub fn get_explorer_address_url(chain: Chain, address: &str) -> String {
        let explorer = Explorer::get_explorer_url(chain);
        format!("{}{}{}", explorer.host, explorer.account_url, address)
    }

    pub fn get_explorer_token_url(chain: Chain, address: &str) -> Option<String> {
        let explorer = Explorer::get_explorer_url(chain);
        if let Some(token_url) = explorer.token_url {
            return Some(format!("{}{}{}", explorer.host, token_url, address));
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bitcoin_explorer_url() {
        let explorer = Explorer::get_explorer_url(Chain::Bitcoin);
        assert_eq!(explorer.host, "https://blockchair.com");
        assert_eq!(explorer.transaction_url, "/bitcoin/transaction/");
        assert_eq!(explorer.account_url, "/bitcoin/address/");
        assert_eq!(explorer.token_url, None);
    }

    #[test]
    fn test_ethereum_explorer_url() {
        let explorer = Explorer::get_explorer_url(Chain::Ethereum);
        assert_eq!(explorer.host, "https://etherscan.io");
        assert_eq!(explorer.transaction_url, "/tx/");
        assert_eq!(explorer.account_url, "/address/");
        assert_eq!(explorer.token_url, Some("/token/"));
    }

    #[test]
    fn test_get_explorer_transaction_url_ethereum() {
        let chain = Chain::Ethereum;
        let transaction_id = "0x7817d35733809af69011e83ece7a9cd593b492c294c85f6a42e0effe6f3f2103";
        let expected_url = "https://etherscan.io/tx/0x7817d35733809af69011e83ece7a9cd593b492c294c85f6a42e0effe6f3f2103";

        let result = Explorer::get_explorer_transaction_url(chain, transaction_id);
        assert_eq!(result, expected_url);
    }

    #[test]
    fn test_get_explorer_transaction_url_bitcoin() {
        let chain = Chain::Bitcoin;
        let transaction_id = "9c723e3ff250b43832df33e96e1a537b9ac08d46809f9e5bcb72e42edeb96f09";
        let expected_url = "https://blockchair.com/bitcoin/transaction/9c723e3ff250b43832df33e96e1a537b9ac08d46809f9e5bcb72e42edeb96f09";

        let result = Explorer::get_explorer_transaction_url(chain, transaction_id);
        assert_eq!(result, expected_url);
    }

    #[test]
    fn test_get_explorer_address_url_ethereum() {
        let chain = Chain::Ethereum;
        let address = "0x1234567890abcdef";
        let expected_url = "https://etherscan.io/address/0x1234567890abcdef";
        assert_eq!(
            Explorer::get_explorer_address_url(chain, address),
            expected_url
        );
    }

    #[test]
    fn test_get_explorer_address_url_bitcoin() {
        let chain = Chain::Bitcoin;
        let address = "1234567890abcdef";
        let expected_url = "https://blockchair.com/bitcoin/address/1234567890abcdef";
        assert_eq!(
            Explorer::get_explorer_address_url(chain, address),
            expected_url
        );
    }

    #[test]
    fn test_get_explorer_token_url_bitcoin() {
        let chain = Chain::Bitcoin;
        assert_eq!(Explorer::get_explorer_token_url(chain, "test"), None);
    }

    #[test]
    fn test_get_explorer_token_url_ethereum() {
        let chain = Chain::Ethereum;
        assert_eq!(
            Explorer::get_explorer_token_url(chain, "0x1234567890abcdef"),
            Some("https://etherscan.io/token/0x1234567890abcdef".to_string())
        );
    }

    #[test]
    fn test_get_explorer_address_url_dymension() {
        let chain = Chain::Dymension;
        assert_eq!(
            Explorer::get_explorer_address_url(chain, "dym1nxswr2xhky3k0rt65paatpzjw8mg5d5rmylu3z"),
            "https://www.mintscan.io/dymension/address/dym1nxswr2xhky3k0rt65paatpzjw8mg5d5rmylu3z"
        );
    }

    #[test]
    fn test_get_explorer_token_url_dymension() {
        let chain = Chain::Dymension;
        let usdc_ibc = "ibc/aWJjL0IzNTA0RTA5MjQ1NkJBNjE4Q0MyOEFDNjcxQTcxRkIwOEM2Q0EwRkQwQkU3QzhBNUI1QTNFMkREOTMzQ0M5RTQ=";
        assert_eq!(
            Explorer::get_explorer_address_url(chain, usdc_ibc),
            "https://www.mintscan.io/dymension/assets/ibc/aWJjL0IzNTA0RTA5MjQ1NkJBNjE4Q0MyOEFDNjcxQTcxRkIwOEM2Q0EwRkQwQkU3QzhBNUI1QTNFMkREOTMzQ0M5RTQ="
        );
    }

    #[test]
    fn test_get_explorer_tx_url_saga() {
        let chain = Chain::Saga;
        assert_eq!(
            Explorer::get_explorer_transaction_url(chain, "DB9E5ABA4574984000533B9522927CEED31814FA94AD39464D54CF2D4EFBF30E"),
            "https://www.mintscan.io/saga/tx/DB9E5ABA4574984000533B9522927CEED31814FA94AD39464D54CF2D4EFBF30E"
        );
    }
}
