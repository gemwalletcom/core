use primitives::Chain;

struct Explorer<'a> {
    #[allow(dead_code)]
    host: &'a str,
    transaction_url: &'a str,
    account_url: &'a str,
    token_url: Option<&'a str>,
}

pub fn get_name_by_host(host: String) -> Option<String> {
    return match host.as_str() {
        "etherscan.io" | "optimistic.etherscan.io" => Some("Etherscan".into()),
        "tonscan.org" => Some("TONScan".into()),
        "solscan.io" => Some("Solscan".into()),
        "opbnbscan.com" => Some("opBNBScan".into()),
        "bscscan.com" | "opbnb.bscscan.com" => Some("BscScan".into()),
        "blockchair.com" => Some("Blockchair".into()),
        "tronscan.org" => Some("TRONSCAN".into()),
        "basescan.org" => Some("BaseScan".into()),
        "blastscan.io" => Some("BlastScan".into()),
        "explorer.sui.io" | "suiexplorer.com" | "suiscan.xyz" => Some("Sui Explorer".into()),
        "explorer.aptoslabs.com" => Some("Aptos Explorer".into()),
        "mintscan.io" | "www.mintscan.io" => Some("MintScan".into()),
        _ => None,
    };
}

pub fn get_explorer_transaction_url(chain: Chain, transaction_id: &str) -> String {
    let explorer = get_explorer(chain);
    format!(
        "{}{}{}",
        explorer.host, explorer.transaction_url, transaction_id
    )
}

pub fn get_explorer_address_url(chain: Chain, address: &str) -> String {
    let explorer = get_explorer(chain);
    format!("{}{}{}", explorer.host, explorer.account_url, address)
}

pub fn get_explorer_token_url(chain: Chain, address: &str) -> Option<String> {
    let explorer = get_explorer(chain);
    if let Some(token_url) = explorer.token_url {
        return Some(format!("{}{}{}", explorer.host, token_url, address));
    }
    None
}

fn get_explorer(chain: Chain) -> Explorer<'static> {
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
        Chain::Binance => Explorer {
            host: "https://explorer.binance.org",
            transaction_url: "/tx/",
            account_url: "/address/",
            token_url: Some("/asset/"),
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_explorer_transaction_url_ethereum() {
        let chain = Chain::Ethereum;
        let transaction_id = "0x7817d35733809af69011e83ece7a9cd593b492c294c85f6a42e0effe6f3f2103";
        let expected_url = "https://etherscan.io/tx/0x7817d35733809af69011e83ece7a9cd593b492c294c85f6a42e0effe6f3f2103";

        let result = get_explorer_transaction_url(chain, transaction_id);
        assert_eq!(result, expected_url);
    }

    #[test]
    fn test_get_explorer_transaction_url_bitcoin() {
        let chain = Chain::Bitcoin;
        let transaction_id = "9c723e3ff250b43832df33e96e1a537b9ac08d46809f9e5bcb72e42edeb96f09";
        let expected_url = "https://blockchair.com/bitcoin/transaction/9c723e3ff250b43832df33e96e1a537b9ac08d46809f9e5bcb72e42edeb96f09";

        let result = get_explorer_transaction_url(chain, transaction_id);
        assert_eq!(result, expected_url);
    }

    #[test]
    fn test_get_explorer_address_url_ethereum() {
        let chain = Chain::Ethereum;
        let address = "0x1234567890abcdef";
        let expected_url = "https://etherscan.io/address/0x1234567890abcdef";
        assert_eq!(get_explorer_address_url(chain, address), expected_url);
    }

    #[test]
    fn test_get_explorer_address_url_bitcoin() {
        let chain = Chain::Bitcoin;
        let address = "1234567890abcdef";
        let expected_url = "https://blockchair.com/bitcoin/address/1234567890abcdef";
        assert_eq!(get_explorer_address_url(chain, address), expected_url);
    }

    #[test]
    fn test_get_explorer_token_url_bitcoin() {
        let chain = Chain::Bitcoin;
        assert_eq!(get_explorer_token_url(chain, "test"), None);
    }
    #[test]
    fn test_get_explorer_token_url_ethereum() {
        let chain = Chain::Ethereum;
        assert_eq!(
            get_explorer_token_url(chain, "0x1234567890abcdef"),
            Some("https://etherscan.io/token/0x1234567890abcdef".to_string())
        );
    }
}
