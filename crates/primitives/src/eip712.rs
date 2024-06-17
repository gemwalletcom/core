use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Codable")]
pub struct EIP712Domain {
    pub name: String,
    pub version: String,
    #[serde(rename = "chainId")]
    pub chain_id: u32,
    #[serde(rename = "verifyingContract")]
    pub verifying_contract: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Codable")]
pub struct EIP712Type {
    pub name: String,
    pub r#type: String,
}

pub fn eip712_domain_types() -> Vec<EIP712Type> {
    vec![
        EIP712Type {
            name: "name".into(),
            r#type: "string".into(),
        },
        EIP712Type {
            name: "version".into(),
            r#type: "string".into(),
        },
        EIP712Type {
            name: "chainId".into(),
            r#type: "uint256".into(),
        },
        EIP712Type {
            name: "verifyingContract".into(),
            r#type: "address".into(),
        },
    ]
}
