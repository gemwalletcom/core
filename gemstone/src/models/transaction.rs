use primitives::{FeeOption, PerpetualDirection, TransferDataOutputType};

pub type GemPerpetualDirection = PerpetualDirection;
pub type GemFeeOption = FeeOption;
pub type GemTransferDataOutputType = TransferDataOutputType;

#[uniffi::remote(Enum)]
pub enum PerpetualDirection {
    Short,
    Long,
}

#[uniffi::remote(Enum)]
pub enum FeeOption {
    TokenAccountCreation,
}

#[uniffi::remote(Enum)]
pub enum TransferDataOutputType {
    EncodedTransaction,
    Signature,
}