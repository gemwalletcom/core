use crate::{AddressName, AddressType, Chain, VerificationStatus};

impl AddressName {
    pub fn mock(address: &str, name: &str, address_type: AddressType, status: VerificationStatus) -> Self {
        Self {
            chain: Chain::Ethereum,
            address: address.to_string(),
            name: name.to_string(),
            address_type: Some(address_type),
            status,
        }
    }
}
