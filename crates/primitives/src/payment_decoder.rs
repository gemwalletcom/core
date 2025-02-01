use crate::{
    asset_id::AssetId,
    erc681::{TransactionRequest, ETHEREUM_SCHEME},
    solana_pay,
    solana_pay::{PayTransfer as SolanaPayTransfer, SOLANA_PAY_SCHEME},
    Chain,
};
use anyhow::{anyhow, Result};
use std::{collections::HashMap, fmt, str::FromStr};

#[derive(Debug, PartialEq)]
pub struct Payment {
    pub address: String,
    pub amount: Option<String>,
    pub memo: Option<String>,
    pub asset_id: Option<AssetId>,
    pub payment_link: Option<LinkType>,
}

#[derive(Debug, PartialEq)]
pub enum LinkType {
    SolanaPay(String),
}

impl fmt::Display for LinkType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LinkType::SolanaPay(link) => write!(f, "{}", link),
        }
    }
}

#[derive(Debug)]
pub struct PaymentURLDecoder;

impl PaymentURLDecoder {
    pub fn decode(string: &str) -> Result<Payment> {
        let chunks: Vec<&str> = string.split(':').collect();

        if chunks.len() == 2 {
            let scheme = chunks[0];
            if scheme == ETHEREUM_SCHEME {
                let transaction_request = TransactionRequest::parse(string)?;
                return Ok(transaction_request.into());
            }
            if scheme == SOLANA_PAY_SCHEME {
                let pay_request = solana_pay::parse(string)?;
                match pay_request {
                    solana_pay::RequestType::Transfer(transfer) => {
                        return Ok(transfer.into());
                    }
                    solana_pay::RequestType::Transaction(link) => {
                        return Ok(Payment {
                            address: "".to_string(),
                            amount: None,
                            memo: None,
                            asset_id: None,
                            payment_link: Some(LinkType::SolanaPay(link)),
                        });
                    }
                }
            }

            let path: &str = chunks[1];
            let path_chunks: Vec<&str> = path.split('?').collect();
            let address = path_chunks[0].to_string();
            let asset_id = Self::decode_scheme(scheme);

            if path_chunks.len() == 1 {
                return Ok(Payment {
                    address,
                    amount: None,
                    memo: None,
                    asset_id,
                    payment_link: None,
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
                    payment_link: None,
                });
            } else {
                return Err(anyhow!("BIP21 format is incorrect"));
            }
        }

        Ok(Payment {
            address: string.to_string(),
            amount: None,
            memo: None,
            asset_id: None,
            payment_link: None,
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
        let memo = val.parameters.get("memo").map(|x| x.to_string());
        let mut chain = Chain::Ethereum;
        if let Some(chain_id) = val.chain_id {
            chain = Chain::from_chain_id(chain_id).unwrap_or(Chain::Ethereum);
        }

        // ERC20
        if val.function_name == Some("transfer".to_string()) {
            address = val.parameters.get("address").map(|x| x.to_string()).unwrap_or("".to_string());
            amount = val.parameters.get("uint256").map(|x| x.to_string());
            asset_id = Some(AssetId::from(chain, Some(val.target_address)));
        } else {
            address = val.target_address;
            amount = val.parameters.get("value").map(|x| x.to_string());
            if amount.is_none() {
                amount = val.parameters.get("amount").map(|x| x.to_string());
            }
            asset_id = Some(AssetId::from(chain, None));
        };
        Self {
            address,
            amount,
            memo,
            asset_id,
            payment_link: None,
        }
    }
}

impl From<SolanaPayTransfer> for Payment {
    fn from(val: SolanaPayTransfer) -> Self {
        Self {
            address: val.recipient,
            amount: val.amount,
            memo: val.memo,
            asset_id: Some(AssetId::from(Chain::Solana, val.spl_token.map(|x| x.to_string()))),
            payment_link: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Chain;

    #[test]
    fn test_address() {
        assert_eq!(
            PaymentURLDecoder::decode("0x1f9090aaE28b8a3dCeaDf281B0F12828e676c326").unwrap(),
            Payment {
                address: "0x1f9090aaE28b8a3dCeaDf281B0F12828e676c326".to_string(),
                amount: None,
                memo: None,
                asset_id: None,
                payment_link: None,
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
                asset_id: None,
                payment_link: None,
            }
        );
        assert_eq!(
            PaymentURLDecoder::decode("solana:HA4hQMs22nCuRN7iLDBsBkboz2SnLM1WkNtzLo6xEDY5?amount=0.266232").unwrap(),
            Payment {
                address: "HA4hQMs22nCuRN7iLDBsBkboz2SnLM1WkNtzLo6xEDY5".to_string(),
                amount: Some("0.266232".to_string()),
                memo: None,
                asset_id: Some(AssetId::from(Chain::Solana, None)),
                payment_link: None,
            }
        );
        assert_eq!(
            PaymentURLDecoder::decode("solana:https%3A%2F%2Fapi.spherepay.co%2Fv1%2Fpublic%2FpaymentLink%2Fpay%2FpaymentLink_1df6564b6b4d43eaa077b732ad9b6ab9%3Fstate%3DAlabama%26country%3DUSA%26lineItems%3D%255B%257B%2522id%2522%253A%2522lineItem_82032b8ea67244e692cd322051e35689%2522%252C%2522quantity%2522%253A500%257D%255D%26solanaPayReference%3D4Vqsq8WhoTbFu8Lw2DbbtnCiHXXmBRN6afF8HkgxrXs7%26paymentReference%3DOZ_UxaOrU_F8fM5GhlrE2%26network%3Dsol%26skipPreflight%3Dfalse").unwrap(),
            Payment {
                address: "".to_string(),
                amount: None,
                memo: None,
                asset_id: None,
                payment_link: Some(LinkType::SolanaPay("https://api.spherepay.co/v1/public/paymentLink/pay/paymentLink_1df6564b6b4d43eaa077b732ad9b6ab9?state=Alabama&country=USA&lineItems=%5B%7B%22id%22%3A%22lineItem_82032b8ea67244e692cd322051e35689%22%2C%22quantity%22%3A500%7D%5D&solanaPayReference=4Vqsq8WhoTbFu8Lw2DbbtnCiHXXmBRN6afF8HkgxrXs7&paymentReference=OZ_UxaOrU_F8fM5GhlrE2&network=sol&skipPreflight=false".to_string())),
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
                payment_link: None,
            }
        );

        assert_eq!(
            PaymentURLDecoder::decode("ethereum:0xA20d8935d61812b7b052E08f0768cFD6D81cB088?amount=0.01233&memo=test").unwrap(),
            Payment {
                address: "0xA20d8935d61812b7b052E08f0768cFD6D81cB088".to_string(),
                amount: Some("0.01233".to_string()),
                memo: Some("test".to_string()),
                asset_id: Some(AssetId::from(Chain::Ethereum, None)),
                payment_link: None,
            }
        );

        assert_eq!(
            PaymentURLDecoder::decode("solana:3u3ta6yXYgpheLGc2GVF3QkLHAUwBrvX71Eg8XXjJHGw?amount=0.42301").unwrap(),
            Payment {
                address: "3u3ta6yXYgpheLGc2GVF3QkLHAUwBrvX71Eg8XXjJHGw".to_string(),
                amount: Some("0.42301".to_string()),
                memo: None,
                asset_id: Some(AssetId::from(Chain::Solana, None)),
                payment_link: None,
            }
        );

        assert_eq!(
            PaymentURLDecoder::decode("ton:EQAzoUpalAaXnVm5MoiYWRZguLFzY0KxFjLv3MkRq5BXzyiQ?amount=0.00001").unwrap(),
            Payment {
                address: "EQAzoUpalAaXnVm5MoiYWRZguLFzY0KxFjLv3MkRq5BXzyiQ".to_string(),
                amount: Some("0.00001".to_string()),
                memo: None,
                asset_id: Some(AssetId::from(Chain::Ton, None)),
                payment_link: None,
            }
        );
    }

    #[test]
    fn test_erc681() {
        assert_eq!(
            PaymentURLDecoder::decode("ethereum:0xcB3028d6120802148f03d6c884D6AD6A210Df62A@1").unwrap(),
            Payment {
                address: "0xcB3028d6120802148f03d6c884D6AD6A210Df62A".to_string(),
                amount: None,
                memo: None,
                asset_id: Some(AssetId::from(Chain::Ethereum, None)),
                payment_link: None,
            }
        );
        assert_eq!(
            PaymentURLDecoder::decode("ethereum:0xcB3028d6120802148f03d6c884D6AD6A210Df62A@0x38?amount=1.23").unwrap(),
            Payment {
                address: "0xcB3028d6120802148f03d6c884D6AD6A210Df62A".to_string(),
                amount: Some("1.23".to_string()),
                memo: None,
                asset_id: Some(AssetId::from(Chain::SmartChain, None)),
                payment_link: None,
            }
        );
    }
}
