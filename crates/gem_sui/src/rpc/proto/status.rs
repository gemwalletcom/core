use gem_encoding::protobuf::proto_decode;

// Field numbers mirror google.rpc.Status:
// https://docs.rs/crate/sui-rpc/0.3.1/source/vendored/proto/google/rpc/status.proto

#[derive(Clone, Debug, Default)]
pub struct Status {
    pub code: i32,
    pub message: String,
}

proto_decode!(Status {
    1 => code: varint_i32,
    2 => message: string,
});
