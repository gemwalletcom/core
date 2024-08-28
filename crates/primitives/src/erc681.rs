use anyhow::{anyhow, Error};
use std::collections::HashMap;

pub const ETHEREUM_SCHEME: &str = "ethereum";
pub const PAY_PREFIX: &str = "pay-";

#[derive(Debug)]
pub struct TransactionRequest {
    pub target_address: String,
    pub prefix: Option<String>,
    pub chain_id: Option<u64>,
    pub function_name: Option<String>,
    pub parameters: HashMap<String, String>,
}

impl TransactionRequest {
    pub fn parse(uri: &str) -> Result<Self, Error> {
        // Split the URI into the scheme and the main part
        let splits = uri.split(':').collect::<Vec<&str>>();
        if splits.len() != 2 {
            return Err(anyhow!("Invalid uri without expected ':'"));
        }

        // Validate the scheme
        let prefix = splits[0];
        if !prefix.eq(ETHEREUM_SCHEME) {
            return Err(anyhow!("Not supported scheme"));
        }

        // Split the main part and the query string
        let parts: Vec<&str> = splits[1].split('?').collect();
        let query_string = if parts.len() > 1 { parts[1] } else { "" };

        // Split the main part by '/'
        let main_parts: Vec<&str> = parts[0].split('/').collect();

        // The first part should be the target address with optional chain id and pay prefix
        let mut target_address = main_parts.first().ok_or(anyhow!("Missing target address"))?.to_string();

        let target_parts = target_address.split('@').collect::<Vec<&str>>();
        let mut chain_id = None;
        if target_parts.len() == 2 {
            if target_parts[1].starts_with("0x") {
                chain_id = u64::from_str_radix(target_parts[1].replace("0x", "").as_str(), 16).ok();
            } else {
                chain_id = target_parts[1].parse().ok();
            }
            target_address = target_parts[0].to_string();
        }

        let mut prefix = None;
        let prefix_parts = target_address.split('-').collect::<Vec<&str>>();
        if prefix_parts.len() == 2 {
            prefix = Some(prefix_parts[0].to_string());
            target_address = prefix_parts[1].to_string();
        }

        // The second part (if exists) is the function name
        let function_name = if main_parts.len() > 1 { Some(main_parts[1].to_string()) } else { None };

        // Parse the query string into key-value pairs
        let mut parameters = HashMap::new();
        let pairs = query_string.split('&');

        for pair in pairs {
            let kv: Vec<&str> = pair.split('=').collect();
            if kv.len() == 2 {
                let key = kv[0].to_string();
                let value = kv[1].to_string();
                parameters.insert(key, value);
            }
        }

        Ok(TransactionRequest {
            target_address,
            prefix,
            chain_id,
            function_name,
            parameters,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_minimal_uri() {
        let uri = "ethereum:0x32Be343B94f860124dC4fEe278FDCBD38C102D88";
        let erc681 = TransactionRequest::parse(uri).unwrap();
        assert_eq!(erc681.target_address, "0x32Be343B94f860124dC4fEe278FDCBD38C102D88");
        assert_eq!(erc681.prefix, None);
        assert_eq!(erc681.chain_id, None);
        assert_eq!(erc681.function_name, None);
        assert_eq!(erc681.parameters.len(), 0);
    }

    #[test]
    fn test_invalid_uri() {
        let uri = "bitcoin:175tWpb8K1S7NmH4Zx6rewF9WQrcZv245W";
        let erc681 = TransactionRequest::parse(uri);
        assert!(erc681.is_err());
    }

    #[test]
    fn test_ens_name_uri() {
        let uri = "ethereum:pay-gemwallet.eth@1";
        let erc681 = TransactionRequest::parse(uri).unwrap();
        assert_eq!(erc681.target_address, "gemwallet.eth");
        assert_eq!(erc681.prefix.unwrap(), "pay");
        assert_eq!(erc681.chain_id, Some(1));
        assert_eq!(erc681.function_name, None);
        assert_eq!(erc681.parameters.len(), 0);
    }

    #[test]
    fn test_chain_id_uri() {
        let uri = "ethereum:pay-0x32Be343B94f860124dC4fEe278FDCBD38C102D88@0x38";
        let erc681 = TransactionRequest::parse(uri).unwrap();
        assert_eq!(erc681.target_address, "0x32Be343B94f860124dC4fEe278FDCBD38C102D88");
        assert_eq!(erc681.prefix.unwrap(), "pay");
        assert_eq!(erc681.chain_id, Some(56));
        assert_eq!(erc681.function_name, None);
        assert_eq!(erc681.parameters.len(), 0);
    }

    #[test]
    fn test_eth_transfer_uri() {
        let uri = "ethereum:0x32Be343B94f860124dC4fEe278FDCBD38C102D88?value=10&gas=200000&gasPrice=20000000000";
        let erc681 = TransactionRequest::parse(uri).unwrap();
        assert_eq!(erc681.target_address, "0x32Be343B94f860124dC4fEe278FDCBD38C102D88");
        assert_eq!(erc681.prefix, None);
        assert_eq!(erc681.chain_id, None);
        assert_eq!(erc681.function_name, None);
        assert_eq!(erc681.parameters.get("value").unwrap(), "10");
        assert_eq!(erc681.parameters.get("gas").unwrap(), "200000");
        assert_eq!(erc681.parameters.get("gasPrice").unwrap(), "20000000000");
    }

    #[test]
    fn test_erc20_transfer_uri() {
        let uri = "ethereum:0x89205a3a3b2a69de6dbf7f01ed13b2108b2c43e7/transfer?address=0x8e23ee67d1332ad560396262c48ffbb01f93d052&uint256=1";
        let erc681 = TransactionRequest::parse(uri).unwrap();
        assert_eq!(erc681.target_address, "0x89205a3a3b2a69de6dbf7f01ed13b2108b2c43e7");
        assert_eq!(erc681.prefix, None);
        assert_eq!(erc681.chain_id, None);
        assert_eq!(erc681.function_name, Some("transfer".to_string()));
        assert_eq!(erc681.parameters.get("address").unwrap(), "0x8e23ee67d1332ad560396262c48ffbb01f93d052");
        assert_eq!(erc681.parameters.get("uint256").unwrap(), "1");
    }
}
