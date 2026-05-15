use gem_encoding::protobuf::proto_encode;

// Field numbers mirror sui-rpc v0.3.1 google.protobuf.FieldMask schema:
// https://docs.rs/crate/sui-rpc/0.3.1/source/vendored/proto/google/protobuf/field_mask.proto

#[derive(Clone, Debug, Default)]
pub struct FieldMask {
    pub paths: Vec<String>,
}

impl FieldMask {
    pub fn from_paths(paths: impl IntoIterator<Item = impl AsRef<str>>) -> Self {
        Self {
            paths: paths.into_iter().map(|path| path.as_ref().to_string()).collect(),
        }
    }

    pub fn from_path_string(paths: &str) -> Self {
        Self::from_paths(paths.split(',').map(str::trim).filter(|path| !path.is_empty()))
    }
}

proto_encode!(FieldMask {
    1 => paths: repeated_string,
});

#[cfg(test)]
mod tests {
    use super::*;
    use gem_encoding::protobuf::MessageEncode;

    #[test]
    fn test_field_mask_encode() {
        assert_eq!(
            FieldMask::from_paths(["digest", "effects.status"]).encode(),
            hex::decode("0a066469676573740a0e656666656374732e737461747573").unwrap()
        );
    }
}
