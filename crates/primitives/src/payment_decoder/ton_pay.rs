use super::error::{PaymentDecoderError, Result};

use crate::{AssetId, Chain};

pub const TON_PAY_SCHEME: &str = "ton";
pub const TON_PAY_TYPE_TRANSFER: &str = "transfer";

#[derive(Debug, Clone)]
pub struct TonPayment {
    pub recipient: String,
    pub asset_id: AssetId,
}

pub fn parse(uri: &str) -> Result<TonPayment> {
    let scheme = format!("{TON_PAY_SCHEME}:");
    if !uri.starts_with(&scheme) {
        return Err(PaymentDecoderError::InvalidScheme);
    }
    let query_part = &uri[scheme.len()..];
    let recipient = extract_address(query_part)?;

    Ok(TonPayment {
        recipient,
        asset_id: AssetId::from_chain(Chain::Ton),
    })
}

fn extract_address(query_part: &str) -> Result<String> {
    let parts: Vec<&str> = query_part.split('/').filter(|s| !s.is_empty()).collect();
    if parts.len() == 2 && parts[0] == TON_PAY_TYPE_TRANSFER {
        Ok(parts[1].to_string())
    } else if parts.len() == 1 {
        Ok(parts[0].to_string())
    } else {
        Err(PaymentDecoderError::InvalidFormat(format!("Invalid URI format: {}", query_part)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_with_transfer() {
        let uri = "ton://transfer/UQA5olhYULHkui4mTQM0LodWG0EqUaxmK6-e3mHrCZFO2diA";
        let payment = parse(uri).unwrap();
        assert_eq!(payment.recipient, "UQA5olhYULHkui4mTQM0LodWG0EqUaxmK6-e3mHrCZFO2diA");
    }

    #[test]
    fn test_parse_without_transfer() {
        let uri = "ton://UQA5olhYULHkui4mTQM0LodWG0EqUaxmK6-e3mHrCZFO2diA";
        let payment = parse(uri).unwrap();
        assert_eq!(payment.recipient, "UQA5olhYULHkui4mTQM0LodWG0EqUaxmK6-e3mHrCZFO2diA");
    }

    #[test]
    fn test_parse_invalid_uri() {
        let uri = "ton://invalid/format";
        assert!(parse(uri).is_err());
    }
}
