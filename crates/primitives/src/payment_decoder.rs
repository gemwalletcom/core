use crate::asset_id::AssetId;
use crate::erc681::{TransactionRequest, ETHEREUM_SCHEME};
use crate::Chain;
use anyhow::Error;
use std::collections::HashMap;
use std::str::FromStr;

#[derive(Debug, PartialEq)]
pub struct Payment {
    pub address: String,
    pub amount: Option<String>,
    pub memo: Option<String>,
    pub asset_id: Option<AssetId>,
}

#[derive(Debug)]
pub struct PaymentURLDecoder;

impl PaymentURLDecoder {
    pub fn decode(string: &str) -> Result<Payment, Error> {
        let chunks: Vec<&str> = string.split(':').collect();

        if chunks.len() == 2 {
            let scheme = chunks[0];
            let path: &str = chunks[1];
            let path_chunks: Vec<&str> = path.split('?').collect();
            let address = if scheme == ETHEREUM_SCHEME {
                let transaction_request = TransactionRequest::parse(path)?;
                return Ok(transaction_request.into());
            } else {
                path_chunks[0].to_string()
            };
            let asset_id = Self::decode_scheme(scheme);

            if path_chunks.len() == 1 {
                return Ok(Payment {
                    address,
                    amount: None,
                    memo: None,
                    asset_id,
                });
            } else if path_chunks.len() == 2 {
                let query = path_chunks[1];
                let params = Self::decode_query_string(query);
                let amount = params.get("amount").cloned();
                let memo = params.get("memo").cloned();

                return Ok(Payment {
                    address,
                    amount,
                    memo,
                    asset_id,
                });
            } else {
                return Err(Error::msg("BIP21 format is incorrect"));
            }
        }

        Ok(Payment {
            address: string.to_string(),
            amount: None,
            memo: None,
            asset_id: None,
        })
    }

    fn decode_query_string(query_string: &str) -> HashMap<String, String> {
        query_string
            .split('&')
            .filter_map(|pair| {
                let components: Vec<&str> = pair.split('=').collect();
                if components.len() == 2 {
                    Some((components[0].to_string(), components[1].to_string()))
                } else {
                    None
                }
            })
            .collect()
    }

    fn decode_scheme(scheme: &str) -> Option<AssetId> {
        let chain = Chain::from_str(scheme).ok()?;
        Some(AssetId::from(chain, None))
    }
}

impl From<TransactionRequest> for Payment {
    fn from(val: TransactionRequest) -> Self {
        let address: String;
        let mut amount: Option<String>;
        let asset_id: Option<AssetId>;
        // ERC20
        if val.function_name == Some("transfer".to_string()) {
            address = val.parameters.get("address").map(|x| x.to_string()).unwrap_or("".to_string());
            amount = val.parameters.get("uint256").map(|x| x.to_string());
            asset_id = Some(AssetId::from(Chain::Ethereum, Some(val.target_address)));
        } else {
            address = val.target_address;
            amount = val.parameters.get("value").map(|x| x.to_string());
            if amount.is_none() {
                amount = val.parameters.get("amount").map(|x| x.to_string());
            }
            asset_id = Some(AssetId::from(Chain::Ethereum, None));
        };
        Self {
            address,
            amount,
            memo: None,
            asset_id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitives::Chain;

    #[test]
    fn test_address() {
        assert_eq!(
            PaymentURLDecoder::decode("0x1f9090aaE28b8a3dCeaDf281B0F12828e676c326").unwrap(),
            Payment {
                address: "0x1f9090aaE28b8a3dCeaDf281B0F12828e676c326".to_string(),
                amount: None,
                memo: None,
                asset_id: None,
            }
        );
    }

    #[test]
    fn test_solana() {
        assert_eq!(
            PaymentURLDecoder::decode("HA4hQMs22nCuRN7iLDBsBkboz2SnLM1WkNtzLo6xEDY5").unwrap(),
            Payment {
                address: "HA4hQMs22nCuRN7iLDBsBkboz2SnLM1WkNtzLo6xEDY5".to_string(),
                amount: None,
                memo: None,
                asset_id: Some(AssetId::from(Chain::Solana, None)),
            }
        );
        assert_eq!(
            PaymentURLDecoder::decode("solana:HA4hQMs22nCuRN7iLDBsBkboz2SnLM1WkNtzLo6xEDY5?amount=0.266232").unwrap(),
            Payment {
                address: "HA4hQMs22nCuRN7iLDBsBkboz2SnLM1WkNtzLo6xEDY5".to_string(),
                amount: Some("0.266232".to_string()),
                memo: None,
                asset_id: Some(AssetId::from(Chain::Solana, None)),
            }
        );
    }

    #[test]
    fn test_bip21() {
        assert_eq!(
            PaymentURLDecoder::decode("bitcoin:bc1pn6pua8a566z7t822kphpd2el45ntm23354c3krfmpe3nnn33lkcskuxrdl?amount=0.00001").unwrap(),
            Payment {
                address: "bc1pn6pua8a566z7t822kphpd2el45ntm23354c3krfmpe3nnn33lkcskuxrdl".to_string(),
                amount: Some("0.00001".to_string()),
                memo: None,
                asset_id: Some(AssetId::from(Chain::Bitcoin, None)),
            }
        );

        assert_eq!(
            PaymentURLDecoder::decode("ethereum:0xA20d8935d61812b7b052E08f0768cFD6D81cB088?amount=0.01233&memo=test").unwrap(),
            Payment {
                address: "0xA20d8935d61812b7b052E08f0768cFD6D81cB088".to_string(),
                amount: Some("0.01233".to_string()),
                memo: Some("test".to_string()),
                asset_id: Some(AssetId::from(Chain::Ethereum, None)),
            }
        );

        assert_eq!(
            PaymentURLDecoder::decode("solana:3u3ta6yXYgpheLGc2GVF3QkLHAUwBrvX71Eg8XXjJHGw?amount=0.42301").unwrap(),
            Payment {
                address: "3u3ta6yXYgpheLGc2GVF3QkLHAUwBrvX71Eg8XXjJHGw".to_string(),
                amount: Some("0.42301".to_string()),
                memo: None,
                asset_id: Some(AssetId::from(Chain::Solana, None)),
            }
        );

        assert_eq!(
            PaymentURLDecoder::decode("ton:EQAzoUpalAaXnVm5MoiYWRZguLFzY0KxFjLv3MkRq5BXzyiQ?amount=0.00001").unwrap(),
            Payment {
                address: "EQAzoUpalAaXnVm5MoiYWRZguLFzY0KxFjLv3MkRq5BXzyiQ".to_string(),
                amount: Some("0.00001".to_string()),
                memo: None,
                asset_id: Some(AssetId::from(Chain::Ton, None)),
            }
        );
    }

    #[test]
    fn test_eip681() {
        assert_eq!(
            PaymentURLDecoder::decode("ethereum:0xcB3028d6120802148f03d6c884D6AD6A210Df62A@0x38").unwrap(),
            Payment {
                address: "0xcB3028d6120802148f03d6c884D6AD6A210Df62A".to_string(),
                amount: None,
                memo: None,
                asset_id: Some(AssetId::from(Chain::Ethereum, None)),
            }
        );
        assert_eq!(
            PaymentURLDecoder::decode("ethereum:0xcB3028d6120802148f03d6c884D6AD6A210Df62A@0x38?amount=1.23").unwrap(),
            Payment {
                address: "0xcB3028d6120802148f03d6c884D6AD6A210Df62A".to_string(),
                amount: Some("1.23".to_string()),
                memo: None,
                asset_id: Some(AssetId::from(Chain::Ethereum, None)),
            }
        );
    }
}
