use super::{FieldMask, Timestamp};
use gem_encoding::protobuf::{proto_decode, proto_encode};

// Field numbers mirror sui-rpc v0.3.1 ledger/epoch/system-state schemas:
// https://docs.rs/crate/sui-rpc/0.3.1/source/vendored/proto/sui/rpc/v2/ledger_service.proto
// https://docs.rs/crate/sui-rpc/0.3.1/source/vendored/proto/sui/rpc/v2/epoch.proto
// https://docs.rs/crate/sui-rpc/0.3.1/source/vendored/proto/sui/rpc/v2/system_state.proto

#[derive(Clone, Debug, Default)]
pub struct GetServiceInfoRequest;

proto_encode!(GetServiceInfoRequest {});

#[derive(Clone, Debug, Default)]
pub struct GetServiceInfoResponse {
    pub chain_id: Option<String>,
    pub chain: Option<String>,
    pub checkpoint_height: Option<u64>,
}

proto_decode!(GetServiceInfoResponse {
    1 => chain_id: optional_string,
    2 => chain: optional_string,
    4 => checkpoint_height: optional_varint_u64,
});

#[derive(Clone, Debug, Default)]
pub struct GetEpochRequest {
    pub read_mask: Option<FieldMask>,
}

impl GetEpochRequest {
    pub fn latest() -> Self {
        Self::default()
    }
}

proto_encode!(GetEpochRequest {
    2 => read_mask: optional_message,
});

#[derive(Clone, Debug, Default)]
pub struct GetEpochResponse {
    pub epoch: Option<Epoch>,
}

proto_decode!(GetEpochResponse {
    1 => epoch: optional_message,
});

#[derive(Clone, Debug, Default)]
pub struct Epoch {
    pub epoch: Option<u64>,
    pub system_state: Option<SystemState>,
    pub start: Option<Timestamp>,
    pub end: Option<Timestamp>,
    pub reference_gas_price: Option<u64>,
}

proto_decode!(Epoch {
    1 => epoch: optional_varint_u64,
    3 => system_state: optional_message,
    6 => start: optional_message,
    7 => end: optional_message,
    8 => reference_gas_price: optional_varint_u64,
});

#[derive(Clone, Debug, Default)]
pub struct SystemState {
    pub validators: Option<ValidatorSet>,
}

proto_decode!(SystemState {
    4 => validators: optional_message,
});

#[derive(Clone, Debug, Default)]
pub struct ValidatorSet {
    pub active_validators: Vec<Validator>,
}

proto_decode!(ValidatorSet {
    2 => active_validators: repeated_message,
});

#[derive(Clone, Debug, Default)]
pub struct Validator {
    pub address: Option<String>,
}

proto_decode!(Validator {
    2 => address: optional_string,
});
