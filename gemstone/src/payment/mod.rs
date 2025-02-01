use anyhow::Error;
use primitives::{payment_decoder::LinkType, Payment, PaymentURLDecoder};

pub mod solana_pay;

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct PaymentWrapper {
    pub address: String,
    pub amount: Option<String>,
    pub memo: Option<String>,
    pub asset_id: Option<String>,
    pub request_link: Option<PaymentLinkType>,
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum PaymentLinkType {
    SolanaPay(String),
}

impl From<LinkType> for PaymentLinkType {
    fn from(value: LinkType) -> Self {
        match value {
            LinkType::SolanaPay(link) => PaymentLinkType::SolanaPay(link),
        }
    }
}

impl From<Payment> for PaymentWrapper {
    fn from(payment: Payment) -> Self {
        PaymentWrapper {
            address: payment.address,
            amount: payment.amount,
            memo: payment.memo,
            asset_id: payment.asset_id.map(|c| c.to_string()),
            request_link: payment.payment_link.map(|x| x.into()),
        }
    }
}

pub fn decode_url(url: &str) -> Result<PaymentWrapper, Error> {
    let payment = PaymentURLDecoder::decode(url)?;
    Ok(payment.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_address() {
        assert_eq!(
            decode_url("solana:3u3ta6yXYgpheLGc2GVF3QkLHAUwBrvX71Eg8XXjJHGw?amount=0.42301").unwrap(),
            PaymentWrapper {
                address: "3u3ta6yXYgpheLGc2GVF3QkLHAUwBrvX71Eg8XXjJHGw".to_string(),
                amount: Some("0.42301".to_string()),
                memo: None,
                asset_id: Some("solana".to_string()),
                request_link: None,
            }
        );
    }
}
