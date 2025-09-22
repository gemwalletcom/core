use primitives::AddressStatus;

pub type GemAddressStatus = AddressStatus;

#[uniffi::remote(Enum)]
pub enum AddressStatus {
    MultiSignature,
}