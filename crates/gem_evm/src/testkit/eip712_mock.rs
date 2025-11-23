use crate::eip712::{EIP712Domain, EIP712Field, EIP712Message, EIP712TypedValue, eip712_domain_types};

impl EIP712Domain {
    pub fn mock(chain_id: u64) -> Self {
        Self {
            name: "Test".to_string(),
            version: Some("1".to_string()),
            chain_id,
            verifying_contract: None,
            salts: None,
        }
    }
}

impl EIP712Message {
    pub fn mock(chain_id: u64) -> Self {
        Self {
            domain: EIP712Domain::mock(chain_id),
            primary_type: "Message".to_string(),
            message: vec![EIP712Field {
                name: "content".to_string(),
                value: EIP712TypedValue::String { value: "Hello".to_string() },
            }],
        }
    }

    pub fn to_json_string(&self) -> String {
        serde_json::to_string(&serde_json::json!({
            "types": {
                "EIP712Domain": eip712_domain_types(),
                "Message": [
                    { "name": "content", "type": "string" }
                ]
            },
            "primaryType": self.primary_type,
            "domain": self.domain,
            "message": {
                "content": "Hello"
            }
        }))
        .unwrap()
    }
}

pub fn mock_eip712_json(chain_id: u64) -> String {
    EIP712Message::mock(chain_id).to_json_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eip712_domain_mock() {
        let domain = EIP712Domain::mock(1);
        assert_eq!(domain.chain_id, 1);
        assert_eq!(domain.name, "Test");
        assert_eq!(domain.version, Some("1".to_string()));
    }

    #[test]
    fn test_eip712_message_mock() {
        let message = EIP712Message::mock(1);
        assert_eq!(message.domain.chain_id, 1);
        assert_eq!(message.primary_type, "Message");
        assert_eq!(message.message.len(), 1);
    }

    #[test]
    fn test_eip712_message_json() {
        let json = mock_eip712_json(1);
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(value["domain"]["chainId"], 1);
        assert_eq!(value["primaryType"], "Message");
    }
}
