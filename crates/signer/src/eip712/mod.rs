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
        let json = r#"
        {
            "types": {
                "EIP712Domain": [
                    { "name": "name", "type": "string" },
                    { "name": "version", "type": "string" },
                    { "name": "chainId", "type": "uint256" },
                    { "name": "verifyingContract", "type": "address" }
                ],
                "Person": [
                    { "name": "name", "type": "string" },
                    { "name": "wallet", "type": "address" }
                ],
                "Mail": [
                    { "name": "from", "type": "Person" },
                    { "name": "to", "type": "Person" },
                    { "name": "contents", "type": "string" }
                ]
            },
            "primaryType": "Mail",
            "domain": {
                "name": "Ether Mail",
                "version": "1",
                "chainId": 1,
                "verifyingContract": "0xCcCCccccCCCCcCCCCCCcCcCccCcCCCcCcccccccC"
            },
            "message": {
                "from": {
                    "name": "Cow",
                    "wallet": "0xCD2a3d9F938E13CD947Ec05AbC7FE734Df8DD826"
                },
                "to": {
                    "name": "Bob",
                    "wallet": "0xbBbBBBBbbBBBbbbBbbBbbbbBBbBbbbbBbBbbBBbB"
                },
                "contents": "Hello, Bob!"
            }
        }
        "#;

        let our_hash = hash_typed_data(json).expect("hash succeeds");
        let expected = <[u8; 32]>::from_hex("be609aee343fb3c4b28e1df9e632fca64fcfaede20f02e86244efddf30957bd2").expect("valid hex");
        assert_eq!(our_hash, expected);
    }

    #[test]
    fn hash_hyperliquid_with_colon_type() {
        let json = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../gem_hypercore/testdata/hl_eip712_approve_agent.json"));
        let digest = hash_typed_data(json).expect("hash succeeds");
        let expected = <[u8; 32]>::from_hex("480af9fd3cdc70c2f8a521388be13620d16a0f643d9cffdfbb65cd019cc27537").expect("valid hex");
        assert_eq!(digest, expected);
    }

    #[test]
    fn hash_handles_arrays_and_nested_types() {
        let json = r#"
        {
            "types": {
                "EIP712Domain": [
                    { "name": "name", "type": "string" },
                    { "name": "version", "type": "string" },
                    { "name": "chainId", "type": "uint256" },
                    { "name": "verifyingContract", "type": "address" }
                ],
                "Inner": [
                    { "name": "flag", "type": "bool" },
                    { "name": "payload", "type": "bytes32" }
                ],
                "Group": [
                    { "name": "members", "type": "address[]" },
                    { "name": "name", "type": "string" },
                    { "name": "nested", "type": "Inner" },
                    { "name": "weights", "type": "uint64[3]" }
                ]
            },
            "primaryType": "Group",
            "domain": {
                "name": "Test",
                "version": "1",
                "chainId": 31337,
                "verifyingContract": "0x0000000000000000000000000000000000000001"
            },
            "message": {
                "members": [
                    "0x90f8bf6a479f320ead074411a4b0e7944ea8c9c1",
                    "0xffcf8fdee72ac11b5c542428b35eef5769c409f0",
                    "0x627306090abab3a6e1400e9345bc60c78a8bef57"
                ],
                "name": "Team Rocket",
                "nested": {
                    "flag": true,
                    "payload": "0x0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20"
                },
                "weights": [ "1", "2", "3" ]
            }
        }
        "#;

        let digest = hash_typed_data(json).expect("hash succeeds");
        let expected = <[u8; 32]>::from_hex("6acbc18af9d2decca3d38571c2f595b1ebb1b93e9e7b046632df71f6ceb217f9").expect("valid hex");
        assert_eq!(digest, expected);
    }

    #[test]
    fn hash_rejects_missing_message() {
        let json = r#"
        {
            "types": {
                "EIP712Domain": [],
                "Simple": [
                    { "name": "value", "type": "uint256" }
                ]
            },
            "primaryType": "Simple",
            "domain": {},
            "message": null
        }
        "#;

        let err = hash_typed_data(json).expect_err("missing message returns error");
        assert!(err.to_string().contains("missing message"));
    }

    #[test]
    fn hash_supports_signed_integers() {
        let json = r#"
        {
            "types": {
                "EIP712Domain": [],
                "Payload": [
                    { "name": "balance", "type": "int256" },
                    { "name": "delta", "type": "int32" },
                    { "name": "active", "type": "bool" }
                ]
            },
            "primaryType": "Payload",
            "domain": {},
            "message": {
                "balance": "-0x0100",
                "delta": -42,
                "active": false
            }
        }
        "#;

        let digest = hash_typed_data(json).expect("hash succeeds");
        let expected = <[u8; 32]>::from_hex("10e6c8b7c51b08488a421a5492d4524439470010eb2f8c80c22b9d918d79a5a9").expect("valid hex");
        assert_eq!(digest, expected);
    }
}
