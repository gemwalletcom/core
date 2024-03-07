use primitives::Chain;

struct Explorer<'a> {
    #[allow(dead_code)]
    host: &'a str,
    transaction_url: &'a str,
    account_url: &'a str,
}

pub fn get_name_by_host(host: String) -> Option<String> {
    return match host.as_str() {
        "etherscan.io" => Some("Etherscan".into()),
        "tonscan.org" => Some("TONScan".into()),
        "solscan.io" => Some("Solscan".into()),
        "opbnbscan.com" => Some("opBNBScan".into()),
        "bscscan.com" => Some("BSCScan".into()),
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

fn get_explorer(chain: Chain) -> Explorer<'static> {
    match chain {
        Chain::Bitcoin => Explorer {
            host: "https://blockchair.com",
            transaction_url: "/bitcoin/transaction/",
            account_url: "/bitcoin/address/",
        },
        Chain::Litecoin => Explorer {
            host: "https://blockchair.com",
            transaction_url: "/litecoin/transaction/",
            account_url: "/litecoin/address/",
        },
        Chain::Ethereum => Explorer {
            host: "https://etherscan.io",
            transaction_url: "/tx/",
            account_url: "/address/",
        },
        Chain::Binance => Explorer {
            host: "https://explorer.binance.org",
            transaction_url: "/tx/",
            account_url: "/address/",
        },
        Chain::SmartChain => Explorer {
            host: "https://bscscan.com",
            transaction_url: "/tx/",
            account_url: "/address/",
        },
        Chain::Solana => Explorer {
            host: "https://solana.fm",
            transaction_url: "/tx/",
            account_url: "/address/",
        },
        Chain::Polygon => Explorer {
            host: "https://polygonscan.com",
            transaction_url: "/tx/",
            account_url: "/address/",
        },
        Chain::Thorchain => Explorer {
            host: "https://viewblock.io/thorchain",
            transaction_url: "/tx/",
            account_url: "/address/",
        },
        Chain::Cosmos => Explorer {
            host: "https://mintscan.io/cosmos",
            transaction_url: "/tx/",
            account_url: "/address/",
        },
        Chain::Osmosis => Explorer {
            host: "https://mintscan.io/osmosis",
            transaction_url: "/tx/",
            account_url: "/address/",
        },
        Chain::Arbitrum => Explorer {
            host: "https://arbiscan.io",
            transaction_url: "/tx/",
            account_url: "/address/",
        },
        Chain::Ton => Explorer {
            host: "https://tonscan.org",
            transaction_url: "/tx/",
            account_url: "/address/",
        },
        Chain::Tron => Explorer {
            host: "https://tronscan.org",
            transaction_url: "/#/transaction/",
            account_url: "/#/address/",
        },
        Chain::Doge => Explorer {
            host: "https://blockchair.com",
            transaction_url: "/dogecoin/transaction/",
            account_url: "/dogecoin/address/",
        },
        Chain::Optimism => Explorer {
            host: "https://optimistic.etherscan.io",
            transaction_url: "/tx/",
            account_url: "/address/",
        },
        Chain::Aptos => Explorer {
            host: "https://explorer.aptoslabs.com",
            transaction_url: "/txn/",
            account_url: "/account/",
        },
        Chain::Base => Explorer {
            host: "https://basescan.org",
            transaction_url: "/tx/",
            account_url: "/address/",
        },
        Chain::AvalancheC => Explorer {
            host: "https://snowtrace.io",
            transaction_url: "/tx/",
            account_url: "/address/",
        },
        Chain::Sui => Explorer {
            host: "https://suiscan.xyz",
            transaction_url: "/mainnet/tx/",
            account_url: "/mainnet/account/",
        },
        Chain::Xrp => Explorer {
            host: "https://xrpscan.com",
            transaction_url: "/tx/",
            account_url: "/account/",
        },
        Chain::OpBNB => Explorer {
            host: "https://opbnb.bscscan.com",
            transaction_url: "/tx/",
            account_url: "/address/",
        },
        Chain::Fantom => Explorer {
            host: "https://ftmscan.com",
            transaction_url: "/tx/",
            account_url: "/address/",
        },
        Chain::Gnosis => Explorer {
            host: "https://gnosisscan.io",
            transaction_url: "/tx/",
            account_url: "/address/",
        },
        Chain::Celestia => Explorer {
            host: "https://mintscan.io/celestia",
            transaction_url: "/tx/",
            account_url: "/address/",
        },
        Chain::Injective => Explorer {
            host: "https://mintscan.io/sei",
            transaction_url: "/tx/",
            account_url: "/address/",
        },
        Chain::Sei => Explorer {
            host: "https://mintscan.io/sei",
            transaction_url: "/txs/",
            account_url: "/address/",
        },
        Chain::Manta => Explorer {
            host: "https://pacific-explorer.manta.network",
            transaction_url: "/tx/",
            account_url: "/address/",
        },
        Chain::Blast => Explorer {
            host: "https://blastscan.io",
            transaction_url: "/tx/",
            account_url: "/address/",
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
}
