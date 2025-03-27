use crate::GemstoneError;
use primitives::{DecodedLinkType, PaymentURLDecoder};

pub mod solana_pay;

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct PaymentWrapper {
    pub address: String,
    pub amount: Option<String>,
    pub memo: Option<String>,
    pub asset_id: Option<String>,
    pub payment_link: Option<String>,
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

/// Exports functions
#[uniffi::export]
pub fn payment_decode_url(string: &str) -> Result<PaymentWrapper, GemstoneError> {
    let payment = PaymentURLDecoder::decode(string)?;
    Ok(PaymentWrapper {
        address: payment.address,
        amount: payment.amount,
        memo: payment.memo,
        asset_id: payment.asset_id.map(|c| c.to_string()),
        payment_link: payment.link.map(|c| c.to_string()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_address() {
        assert_eq!(
            payment_decode_url("solana:3u3ta6yXYgpheLGc2GVF3QkLHAUwBrvX71Eg8XXjJHGw?amount=0.42301").unwrap(),
            PaymentWrapper {
                address: "3u3ta6yXYgpheLGc2GVF3QkLHAUwBrvX71Eg8XXjJHGw".to_string(),
                amount: Some("0.42301".to_string()),
                memo: None,
                asset_id: Some("solana".to_string()),
                payment_link: None,
            }
        );
    }
}
