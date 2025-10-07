use gem_swapper::SwapperError;
use primitives::Chain;

type Permit2Data = gem_swapper::permit2_data::Permit2Data;
type PermitSingle = gem_swapper::permit2_data::PermitSingle;
type Permit2Detail = gem_swapper::permit2_data::Permit2Detail;

pub type Permit2ApprovalData = gem_swapper::models::Permit2ApprovalData;

#[uniffi::remote(Record)]
pub struct Permit2Detail {
    pub token: String,
    pub amount: String,
    pub expiration: u64,
    pub nonce: u64,
}

#[uniffi::remote(Record)]
pub struct PermitSingle {
    pub details: Permit2Detail,
    pub spender: String,
    pub sig_deadline: u64,
}

#[uniffi::remote(Record)]
pub struct Permit2Data {
    pub permit_single: PermitSingle,
    pub signature: Vec<u8>,
}

#[uniffi::remote(Record)]
pub struct Permit2ApprovalData {
    pub token: String,
    pub spender: String,
    pub value: String,
    pub permit2_contract: String,
    pub permit2_nonce: u64,
}

#[uniffi::export]
pub fn permit2_data_to_eip712_json(chain: Chain, data: PermitSingle, contract: &str) -> Result<String, SwapperError> {
    gem_swapper::permit2_data::permit2_data_to_eip712_json(chain, data, contract)
}
