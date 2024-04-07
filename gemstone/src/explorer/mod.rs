use primitives::{Chain, Explorer};

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
    Explorer::get_explorer_transaction_url(chain, transaction_id)
}

pub fn get_explorer_address_url(chain: Chain, address: &str) -> String {
    Explorer::get_explorer_address_url(chain, address)
}

pub fn get_explorer_token_url(chain: Chain, address: &str) -> Option<String> {
    Explorer::get_explorer_token_url(chain, address)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_name_by_host_ethereum() {
        let host = "etherscan.io".to_string();
        let expected = Some("Etherscan".into());
        let result = get_name_by_host(host);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_get_explorer_transaction_url_ethereum() {
        let chain = Chain::Ethereum;
        let transaction_id = "123456";
        let expected = "https://etherscan.io/tx/123456".to_string();
        let result = get_explorer_transaction_url(chain, transaction_id);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_get_explorer_address_url_ethereum() {
        let chain = Chain::Ethereum;
        let address = "0x1234567890abcdef";
        let expected = "https://etherscan.io/address/0x1234567890abcdef".to_string();
        let result = get_explorer_address_url(chain, address);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_get_explorer_token_url_ethereum() {
        let chain = Chain::Ethereum;
        let address = "0x1234567890abcdef";
        let expected = Some("https://etherscan.io/token/0x1234567890abcdef".to_string());
        let result = get_explorer_token_url(chain, address);
        assert_eq!(result, expected);
    }
}
