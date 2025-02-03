use anyhow::Error;
use primitives::{payment_decoder::DecodedLinkType, Payment, PaymentURLDecoder};

pub mod solana_pay;

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum PaymentType {
    Payment(PaymentWrapper),
    PaymentLink(PaymentLinkType),
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct PaymentWrapper {
    pub address: String,
    pub amount: Option<String>,
    pub memo: Option<String>,
    pub asset_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum PaymentLinkType {
    SolanaPay(String),
}

impl From<DecodedLinkType> for PaymentLinkType {
    fn from(value: DecodedLinkType) -> Self {
        match value {
            DecodedLinkType::SolanaPay(link) => PaymentLinkType::SolanaPay(link),
        }
    }
}

impl From<Payment> for PaymentType {
    fn from(payment: Payment) -> Self {
        if let Some(link) = payment.link {
            return PaymentType::PaymentLink(link.into());
        }

        PaymentType::Payment(PaymentWrapper {
            address: payment.address,
            amount: payment.amount,
            memo: payment.memo,
            asset_id: payment.asset_id.map(|c| c.to_string()),
        })
    }
}

pub fn decode_url(url: &str) -> Result<PaymentType, Error> {
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
            PaymentType::Payment(PaymentWrapper {
                address: "3u3ta6yXYgpheLGc2GVF3QkLHAUwBrvX71Eg8XXjJHGw".to_string(),
                amount: Some("0.42301".to_string()),
                memo: None,
                asset_id: Some("solana".to_string()),
            })
        );
    }
}
