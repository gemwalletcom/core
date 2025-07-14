#![allow(non_snake_case)]
use serde::{Deserialize, Serialize};
#[derive(Debug, PartialEq)]
pub struct Permit {
    pub value: String,
    pub deadline: String,
    pub v: u8,
    pub r: Vec<u8>,
    pub s: Vec<u8>,
}

use crate::eip712::{EIP712Domain, EIP712Type};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ERC2612Permit {
    pub owner: String,
    pub spender: String,
    pub value: String,
    pub nonce: String,
    pub deadline: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ERC2612Types {
    #[serde(rename = "EIP712Domain")]
    pub eip712Domain: Vec<EIP712Type>,
    #[serde(rename = "Permit")]
    pub permit: Vec<EIP712Type>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ERC2612PermitMessage {
    pub types: ERC2612Types,
    #[serde(rename = "primaryType")]
    pub primary_type: String,
    pub domain: EIP712Domain,
    pub message: ERC2612Permit,
}

pub fn erc2612_permit_types() -> Vec<EIP712Type> {
    vec![
        EIP712Type {
            name: "owner".into(),
            r#type: "address".into(),
        },
        EIP712Type {
            name: "spender".into(),
            r#type: "address".into(),
        },
        EIP712Type {
            name: "value".into(),
            r#type: "uint256".into(),
        },
        EIP712Type {
            name: "nonce".into(),
            r#type: "uint256".into(),
        },
        EIP712Type {
            name: "deadline".into(),
            r#type: "uint256".into(),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eip712::*;

    #[test]
    fn test_erc2612_permit_message_json() {
        let eip712_domain_types = eip712_domain_types();
        let permit_types = erc2612_permit_types();

        let permit = ERC2612Permit {
            owner: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".to_string(),
            spender: "0x889edC2eDab5f40e902b864aD4d7AdE8E412F9B1".to_string(),
            value: "11005735849467938".to_string(),
            nonce: "1".to_string(),
            deadline: "1718895600".to_string(),
        };

        let message = ERC2612PermitMessage {
            types: ERC2612Types {
                eip712Domain: eip712_domain_types,
                permit: permit_types,
            },
            primary_type: "Permit".to_string(),
            domain: EIP712Domain {
                name: "Liquid staked Ether 2.0".to_string(),
                version: Some("2".to_string()),
                chain_id: 1,
                verifying_contract: Some("0xae7ab96520DE3A18E5e111B5EaAb095312D7fE84".to_string()),
                salts: None,
            },
            message: permit,
        };

        let expected = r#"
        {
            "domain": {
                "verifyingContract": "0xae7ab96520DE3A18E5e111B5EaAb095312D7fE84",
                "chainId": 1,
                "version": "2",
                "name": "Liquid staked Ether 2.0"
            },
            "message": {
                "nonce": "1",
                "value": "11005735849467938",
                "deadline": "1718895600",
                "spender": "0x889edC2eDab5f40e902b864aD4d7AdE8E412F9B1",
                "owner": "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7"
            },
            "primaryType": "Permit",
            "types": {
                "Permit": [
                    {
                        "type": "address",
                        "name": "owner"
                    },
                    {
                        "name": "spender",
                        "type": "address"
                    },
                    {
                        "type": "uint256",
                        "name": "value"
                    },
                    {
                        "type": "uint256",
                        "name": "nonce"
                    },
                    {
                        "name": "deadline",
                        "type": "uint256"
                    }
                ],
                "EIP712Domain": [
                    {
                        "type": "string",
                        "name": "name"
                    },
                    {
                        "name": "version",
                        "type": "string"
                    },
                    {
                        "name": "chainId",
                        "type": "uint256"
                    },
                    {
                        "type": "address",
                        "name": "verifyingContract"
                    }
                ]
            }
        }
        "#;

        assert_eq!(message, serde_json::from_str::<ERC2612PermitMessage>(expected).unwrap());
    }
}
