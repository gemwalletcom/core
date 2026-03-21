mod data;
mod hash_impl;
mod parse;

pub use hash_impl::hash_typed_data;

#[cfg(test)]
mod tests {
    use super::*;
    use hex::FromHex;

    #[test]
    fn hash_matches_reference_vector() {
        let json = include_str!("../../testdata/eip712_reference_vector.json");

        let our_hash = hash_typed_data(json).expect("hash succeeds");
        let expected = <[u8; 32]>::from_hex("be609aee343fb3c4b28e1df9e632fca64fcfaede20f02e86244efddf30957bd2").unwrap();
        assert_eq!(our_hash, expected);
    }

    #[test]
    fn hash_hyperliquid_with_colon_type() {
        let json = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../gem_hypercore/testdata/hl_eip712_approve_agent.json"));
        let digest = hash_typed_data(json).expect("hash succeeds");
        let expected = <[u8; 32]>::from_hex("480af9fd3cdc70c2f8a521388be13620d16a0f643d9cffdfbb65cd019cc27537").unwrap();
        assert_eq!(digest, expected);
    }

    #[test]
    fn hash_handles_arrays_and_nested_types() {
        let json = include_str!("../../testdata/eip712_arrays_nested.json");

        let digest = hash_typed_data(json).expect("hash succeeds");
        let expected = <[u8; 32]>::from_hex("6acbc18af9d2decca3d38571c2f595b1ebb1b93e9e7b046632df71f6ceb217f9").unwrap();
        assert_eq!(digest, expected);
    }

    #[test]
    fn hash_rejects_missing_message() {
        let json = include_str!("../../testdata/eip712_missing_message.json");

        let err = hash_typed_data(json).expect_err("missing message returns error");
        assert!(err.to_string().contains("missing message"));
    }

    #[test]
    fn hash_supports_signed_integers() {
        let json = include_str!("../../testdata/eip712_signed_integers.json");

        let digest = hash_typed_data(json).expect("hash succeeds");
        let expected = <[u8; 32]>::from_hex("10e6c8b7c51b08488a421a5492d4524439470010eb2f8c80c22b9d918d79a5a9").unwrap();
        assert_eq!(digest, expected);
    }
}
