use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PhantomAgent {
    pub source: String,
    #[serde(rename = "connectionId")]
    pub connection_id: String,
}

impl PhantomAgent {
    pub fn new(action_hash: String) -> Self {
        Self {
            source: "a".to_string(),
            connection_id: format!("0x{action_hash}"),
        }
    }
}
