use num_bigint::BigUint;
use primitives::{SignerError, SignerInput, YieldProvider};

use super::request::TransferRequest;
use crate::{signer::cells::BagOfCells, tonstakers};

pub(super) fn build_request(input: &SignerInput) -> Result<TransferRequest, SignerError> {
    let earn_data = input.input_type.get_earn_data()?;
    if earn_data.call_data.is_empty() {
        return Err(SignerError::invalid_input("earn call data is required"));
    }
    let payload = BagOfCells::parse_base64_root(&earn_data.call_data)?;
    let attached_value = attached_value(input)?;
    TransferRequest::new_with_payload(&earn_data.contract_address, &attached_value.to_string(), input.memo.clone(), Some(payload), true, None)
}

fn attached_value(input: &SignerInput) -> Result<BigUint, SignerError> {
    let earn_type = input.input_type.get_earn_type()?;
    let provider_id = earn_type.provider_id();
    if provider_id == YieldProvider::Tonstakers.as_ref() {
        return tonstakers::attached_value(earn_type, &input.value);
    }
    Err(SignerError::invalid_input(format!("unsupported TON earn provider: {provider_id}")))
}
