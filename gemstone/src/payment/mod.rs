use anyhow::Error;

use primitives::{Payment, PaymentURLDecoder};

#[derive(uniffi::Record, Debug, Clone, PartialEq)]
pub struct PaymentWrapper {
    pub address: String,
    pub amount: Option<String>,
    pub memo: Option<String>,
    pub asset_id: Option<String>,
}

impl PaymentWrapper {
    fn from_primitive(payment: Payment) -> Self {
        PaymentWrapper {
            address: payment.address,
            amount: payment.amount,
            memo: payment.memo,
            asset_id: payment.asset_id.map(|c| c.to_string()),
        }
    }
}

pub fn decode_url(url: &str) -> Result<PaymentWrapper, Error> {
    let payment = PaymentURLDecoder::decode(url)?;
    Ok(PaymentWrapper::from_primitive(payment))
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
            }
        );
    }
}
