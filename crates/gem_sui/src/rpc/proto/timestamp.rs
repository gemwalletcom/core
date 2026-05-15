use gem_encoding::protobuf::proto_decode;

// Field numbers mirror sui-rpc v0.3.1 google.protobuf.Timestamp schema:
// https://docs.rs/crate/sui-rpc/0.3.1/source/vendored/proto/google/protobuf/timestamp.proto

#[derive(Clone, Debug, Default)]
pub struct Timestamp {
    pub seconds: i64,
    pub nanos: i32,
}

impl Timestamp {
    pub fn millis(&self) -> i64 {
        self.seconds.saturating_mul(1000) + i64::from(self.nanos / 1_000_000)
    }
}

proto_decode!(Timestamp {
    1 => seconds: varint_i64,
    2 => nanos: varint_i32,
});
