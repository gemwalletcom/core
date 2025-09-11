use crate::{asset_id::AssetId, Chain};
use super::error::{PaymentDecoderError, Result};
use std::{collections::HashMap, fmt, str::FromStr};

use super::{
    erc681::{TransactionRequest, ETHEREUM_SCHEME},
    solana_pay::{self, PayTransfer as SolanaPayTransfer, SOLANA_PAY_SCHEME},
    ton_pay::{self, TON_PAY_SCHEME},
};

#[derive(Debug, PartialEq)]
pub struct Payment {
    pub address: String,
    pub amount: Option<String>,
    pub memo: Option<String>,
    pub asset_id: Option<AssetId>,
    pub link: Option<DecodedLinkType>,
}

impl Payment {
    pub fn new_address(address: &str) -> Self {
        Self {
            address: address.to_string(),
            amount: None,
            memo: None,
            asset_id: None,
            link: None,
        }
    }

    pub fn new_link(link: DecodedLinkType) -> Self {
        Self {
            address: "".to_string(),
            amount: None,
            memo: None,
            asset_id: None,
            link: Some(link),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum DecodedLinkType {
    SolanaPay(String),
}

impl fmt::Display for DecodedLinkType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DecodedLinkType::SolanaPay(link) => write!(f, "{link}"),
        }
    }
}

#[derive(Debug)]
pub struct PaymentURLDecoder;

impl PaymentURLDecoder {
    pub fn decode(string: &str) -> Result<Payment> {
        let chunks: Vec<&str> = string.split(':').collect();

        match chunks.len() {
            // Handle case with no scheme
            1 => {
                // Check for query parameters
                if string.contains('?') {
                    let parts: Vec<&str> = string.splitn(2, '?').collect();
                    if parts.len() == 2 {
                        let address = parts[0].to_string();
                        let params = Self::decode_query_string(parts[1]);
                        return Ok(Payment {
                            address,
                            amount: params.get("amount").cloned(),
                            memo: params.get("memo").cloned(),
                            asset_id: None,
                            link: None,
                        });
                    }
                }
                // No scheme and no query parameters
                Ok(Payment::new_address(string))
            }
            // Handle case with scheme
            2 => {
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
                                link: Some(DecodedLinkType::SolanaPay(link)),
                            });
                        }
                    }
                }
                if scheme == TON_PAY_SCHEME {
                    let ton_payment = ton_pay::parse(string)?;
                    return Ok(Payment {
                        address: ton_payment.recipient,
                        amount: None,
                        memo: None,
                        asset_id: Some(ton_payment.asset_id),
                        link: None,
                    });
                }

                let path: &str = chunks[1];
                let path_chunks: Vec<&str> = path.split('?').collect();
                let address = path_chunks[0].to_string();
                let asset_id = Self::decode_scheme(scheme);

                if path_chunks.len() == 1 {
                    Ok(Payment {
                        address,
                        amount: None,
                        memo: None,
                        asset_id,
                        link: None,
                    })
                } else if path_chunks.len() == 2 {
                    let query = path_chunks[1];
                    let params = Self::decode_query_string(query);
                    let amount = params.get("amount").cloned();
                    let memo = params.get("memo").cloned();

                    Ok(Payment {
                        address,
                        amount,
                        memo,
                        asset_id,
                        link: None,
                    })
                } else {
                    Err(PaymentDecoderError::InvalidFormat("BIP21 format is incorrect".to_string()))
                }
            }
            // Handle any other case (shouldn't normally happen)
            _ => Ok(Payment::new_address(string)),
        }
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
            link: None,
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
            link: None,
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
            Payment::new_address("0x1f9090aaE28b8a3dCeaDf281B0F12828e676c326")
        );
    }

    #[test]
    fn test_solana() {
        assert_eq!(
            PaymentURLDecoder::decode("HA4hQMs22nCuRN7iLDBsBkboz2SnLM1WkNtzLo6xEDY5").unwrap(),
            Payment::new_address("HA4hQMs22nCuRN7iLDBsBkboz2SnLM1WkNtzLo6xEDY5")
        );
        assert_eq!(
            PaymentURLDecoder::decode("solana:HA4hQMs22nCuRN7iLDBsBkboz2SnLM1WkNtzLo6xEDY5?amount=0.266232").unwrap(),
            Payment {
                address: "HA4hQMs22nCuRN7iLDBsBkboz2SnLM1WkNtzLo6xEDY5".to_string(),
                amount: Some("0.266232".to_string()),
                memo: None,
                asset_id: Some(AssetId::from_chain(Chain::Solana)),
                link: None,
            }
        );
        assert_eq!(
            PaymentURLDecoder::decode("solana:https%3A%2F%2Fapi.spherepay.co%2Fv1%2Fpublic%2FpaymentLink%2Fpay%2FpaymentLink_1df6564b6b4d43eaa077b732ad9b6ab9%3Fstate%3DAlabama%26country%3DUSA%26lineItems%3D%255B%257B%2522id%2522%253A%2522lineItem_82032b8ea67244e692cd322051e35689%2522%252C%2522quantity%2522%253A500%257D%255D%26solanaPayReference%3D4Vqsq8WhoTbFu8Lw2DbbtnCiHXXmBRN6afF8HkgxrXs7%26paymentReference%3DOZ_UxaOrU_F8fM5GhlrE2%26network%3Dsol%26skipPreflight%3Dfalse").unwrap(),
            Payment::new_link(DecodedLinkType::SolanaPay("https://api.spherepay.co/v1/public/paymentLink/pay/paymentLink_1df6564b6b4d43eaa077b732ad9b6ab9?state=Alabama&country=USA&lineItems=%5B%7B%22id%22%3A%22lineItem_82032b8ea67244e692cd322051e35689%22%2C%22quantity%22%3A500%7D%5D&solanaPayReference=4Vqsq8WhoTbFu8Lw2DbbtnCiHXXmBRN6afF8HkgxrXs7&paymentReference=OZ_UxaOrU_F8fM5GhlrE2&network=sol&skipPreflight=false".to_string())),
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
                asset_id: Some(AssetId::from_chain(Chain::Bitcoin)),
                link: None,
            }
        );

        assert_eq!(
            PaymentURLDecoder::decode("ethereum:0xA20d8935d61812b7b052E08f0768cFD6D81cB088?amount=0.01233&memo=test").unwrap(),
            Payment {
                address: "0xA20d8935d61812b7b052E08f0768cFD6D81cB088".to_string(),
                amount: Some("0.01233".to_string()),
                memo: Some("test".to_string()),
                asset_id: Some(AssetId::from_chain(Chain::Ethereum)),
                link: None,
            }
        );

        assert_eq!(
            PaymentURLDecoder::decode("solana:3u3ta6yXYgpheLGc2GVF3QkLHAUwBrvX71Eg8XXjJHGw?amount=0.42301").unwrap(),
            Payment {
                address: "3u3ta6yXYgpheLGc2GVF3QkLHAUwBrvX71Eg8XXjJHGw".to_string(),
                amount: Some("0.42301".to_string()),
                memo: None,
                asset_id: Some(AssetId::from_chain(Chain::Solana)),
                link: None,
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
                asset_id: Some(AssetId::from_chain(Chain::Ethereum)),
                link: None,
            }
        );
        assert_eq!(
            PaymentURLDecoder::decode("ethereum:0xcB3028d6120802148f03d6c884D6AD6A210Df62A@0x38?amount=1.23").unwrap(),
            Payment {
                address: "0xcB3028d6120802148f03d6c884D6AD6A210Df62A".to_string(),
                amount: Some("1.23".to_string()),
                memo: None,
                asset_id: Some(AssetId::from_chain(Chain::SmartChain)),
                link: None,
            }
        );
    }

    #[test]
    fn test_ton_address() {
        assert_eq!(
            PaymentURLDecoder::decode("UQA5olhYULHkui4mTQM0LodWG0EqUaxmK6-e3mHrCZFO2diA").unwrap(),
            Payment {
                address: "UQA5olhYULHkui4mTQM0LodWG0EqUaxmK6-e3mHrCZFO2diA".to_string(),
                amount: None,
                memo: None,
                asset_id: None,
                link: None,
            }
        );
        assert_eq!(
            PaymentURLDecoder::decode("ton://transfer/UQA5olhYULHkui4mTQM0LodWG0EqUaxmK6-e3mHrCZFO2diA").unwrap(),
            Payment {
                address: "UQA5olhYULHkui4mTQM0LodWG0EqUaxmK6-e3mHrCZFO2diA".to_string(),
                amount: None,
                memo: None,
                asset_id: Some(AssetId::from_chain(Chain::Ton)),
                link: None,
            }
        );
    }

    #[test]
    fn test_address_with_amount() {
        assert_eq!(
            PaymentURLDecoder::decode("0x25851Bf7D35293A89F710eBFbD4718322eF7B174?amount=50.72").unwrap(),
            Payment {
                address: "0x25851Bf7D35293A89F710eBFbD4718322eF7B174".to_string(),
                amount: Some("50.72".to_string()),
                memo: None,
                asset_id: None,
                link: None,
            }
        );
    }
}
