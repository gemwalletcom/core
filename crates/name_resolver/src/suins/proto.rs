use gem_encoding::protobuf::{MessageDecode, MessageEncode, MessageResult, proto_decode, proto_encode};

// Field numbers mirror sui-rpc v0.3.1 SuiNS schema:
// https://docs.rs/crate/sui-rpc/0.3.1/source/vendored/proto/sui/rpc/v2/name_service.proto

pub(super) fn encode_lookup_name_request(name: &str) -> Vec<u8> {
    LookupNameRequest { name: Some(name.to_string()) }.encode()
}

pub(super) fn decode_lookup_name_response(data: &[u8]) -> MessageResult<String> {
    LookupNameResponse::decode(data)?
        .record
        .and_then(|record| record.target_address)
        .ok_or_else(|| "SuiNS record has no target address".into())
}

#[derive(Clone, Debug, Default)]
struct LookupNameRequest {
    name: Option<String>,
}

proto_encode!(LookupNameRequest {
    1 => name: optional_string,
});

#[derive(Clone, Debug, Default)]
struct LookupNameResponse {
    record: Option<NameRecord>,
}

proto_decode!(LookupNameResponse {
    1 => record: optional_message,
});

#[derive(Clone, Debug, Default)]
struct NameRecord {
    target_address: Option<String>,
}

proto_decode!(NameRecord {
    5 => target_address: optional_string,
});

#[cfg(test)]
mod tests {
    use super::*;
    use gem_encoding::protobuf::{decode_grpc_frame, encode_bytes_field, encode_grpc_frame, encode_string_field};

    #[test]
    fn test_encode_lookup_name_request() {
        assert_eq!(encode_lookup_name_request("alpha.sui"), encode_string_field(1, "alpha.sui"));
    }

    #[test]
    fn test_decode_lookup_name_response() {
        let target = "0x54e5c2a6f1276ac2ff623ac54e53e5a61a576906b3ec42fac8fe8bf5615d0957";
        let record = [
            encode_string_field(1, "record-id"),
            encode_string_field(2, "alpha.sui"),
            encode_string_field(5, target),
            encode_bytes_field(6, &[encode_string_field(1, "avatar"), encode_string_field(2, "ipfs://avatar")].concat()),
        ]
        .concat();
        let response = encode_bytes_field(1, &record);

        assert_eq!(decode_lookup_name_response(&response).unwrap(), target);
    }

    #[test]
    fn test_decode_lookup_name_response_rejects_missing_target() {
        let record = encode_string_field(2, "alpha.sui");
        let response = encode_bytes_field(1, &record);

        assert_eq!(decode_lookup_name_response(&response).unwrap_err().to_string(), "SuiNS record has no target address");
    }

    #[test]
    fn test_decode_grpc_message_rejects_truncated_frame() {
        let payload = encode_string_field(1, "alpha.sui");
        let mut frame = vec![0];
        frame.extend_from_slice(&((payload.len() + 1) as u32).to_be_bytes());
        frame.extend_from_slice(&payload);

        assert_eq!(decode_grpc_frame(&frame).unwrap_err().to_string(), "truncated gRPC response frame");
    }

    #[test]
    fn test_decode_grpc_message_roundtrip() {
        let payload = encode_lookup_name_request("alpha.sui");

        assert_eq!(decode_grpc_frame(&encode_grpc_frame(&payload)).unwrap(), payload);
    }
}
