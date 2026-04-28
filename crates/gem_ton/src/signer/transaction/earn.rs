use primitives::{SignerError, SignerInput};

use super::request::TransferRequest;
use crate::{signer::cells::BagOfCells, tonstakers};

pub(super) fn build_request(input: &SignerInput) -> Result<TransferRequest, SignerError> {
    let earn_data = input.input_type.get_earn_data()?;
    if earn_data.call_data.is_empty() {
        return Err(SignerError::invalid_input("earn call data is required"));
    }
    let payload = BagOfCells::parse_base64_root(&earn_data.call_data)?;
    let earn_type = input.input_type.get_earn_type()?;
    let attached_value = tonstakers::attached_value(earn_type, &input.value)?;
    TransferRequest::new_with_payload(&earn_data.contract_address, &attached_value.to_string(), input.memo.clone(), Some(payload), true, None)
}
