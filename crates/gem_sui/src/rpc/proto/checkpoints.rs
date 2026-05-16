use super::{FieldMask, Timestamp};
use gem_encoding::protobuf::{proto_decode, proto_encode};

// Field numbers mirror sui-rpc v0.3.1 ledger/checkpoint schemas:
// https://docs.rs/crate/sui-rpc/0.3.1/source/vendored/proto/sui/rpc/v2/ledger_service.proto
// https://docs.rs/crate/sui-rpc/0.3.1/source/vendored/proto/sui/rpc/v2/checkpoint.proto
// https://docs.rs/crate/sui-rpc/0.3.1/source/vendored/proto/sui/rpc/v2/checkpoint_summary.proto
// https://docs.rs/crate/sui-rpc/0.3.1/source/vendored/proto/sui/rpc/v2/checkpoint_contents.proto

#[derive(Clone, Debug, Default)]
pub struct GetCheckpointRequest {
    pub sequence_number: Option<u64>,
    pub read_mask: Option<FieldMask>,
}

impl GetCheckpointRequest {
    pub fn by_sequence_number(sequence_number: u64) -> Self {
        Self {
            sequence_number: Some(sequence_number),
            read_mask: None,
        }
    }
}

proto_encode!(GetCheckpointRequest {
    1 => sequence_number: optional_varint_u64,
    3 => read_mask: optional_message,
});

#[derive(Clone, Debug, Default)]
pub struct GetCheckpointResponse {
    pub checkpoint: Option<Checkpoint>,
}

proto_decode!(GetCheckpointResponse {
    1 => checkpoint: optional_message,
});

#[derive(Clone, Debug, Default)]
pub struct Checkpoint {
    pub sequence_number: Option<u64>,
    pub digest: Option<String>,
    pub summary: Option<CheckpointSummary>,
    pub contents: Option<CheckpointContents>,
}

proto_decode!(Checkpoint {
    1 => sequence_number: optional_varint_u64,
    2 => digest: optional_string,
    3 => summary: optional_message,
    5 => contents: optional_message,
});

#[derive(Clone, Debug, Default)]
pub struct CheckpointSummary {
    pub epoch: Option<u64>,
    pub total_network_transactions: Option<u64>,
    pub previous_digest: Option<String>,
    pub timestamp: Option<Timestamp>,
}

proto_decode!(CheckpointSummary {
    3 => epoch: optional_varint_u64,
    5 => total_network_transactions: optional_varint_u64,
    7 => previous_digest: optional_string,
    9 => timestamp: optional_message,
});

#[derive(Clone, Debug, Default)]
pub struct CheckpointContents {
    pub transactions: Vec<CheckpointedTransactionInfo>,
}

proto_decode!(CheckpointContents {
    4 => transactions: repeated_message,
});

#[derive(Clone, Debug, Default)]
pub struct CheckpointedTransactionInfo {
    pub transaction: Option<String>,
}

proto_decode!(CheckpointedTransactionInfo {
    1 => transaction: optional_string,
});

#[cfg(test)]
mod tests {
    use super::*;
    use gem_encoding::protobuf::MessageDecode;

    #[test]
    fn test_checkpoint_response_wire_bytes_decode() {
        let response = hex::decode("0a2a082a1211636865636b706f696e742d6469676573741a0a180728633a04707265762a0722050a03747831").unwrap();
        let checkpoint = GetCheckpointResponse::decode(&response).unwrap().checkpoint.unwrap();
        let summary = checkpoint.summary.unwrap();
        let contents = checkpoint.contents.unwrap();

        assert_eq!(checkpoint.sequence_number, Some(42));
        assert_eq!(checkpoint.digest.as_deref(), Some("checkpoint-digest"));
        assert_eq!(summary.epoch, Some(7));
        assert_eq!(summary.total_network_transactions, Some(99));
        assert_eq!(summary.previous_digest.as_deref(), Some("prev"));
        assert_eq!(contents.transactions[0].transaction.as_deref(), Some("tx1"));
    }
}
