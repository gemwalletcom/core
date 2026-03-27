use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PerpDex {
    pub name: String,
    pub is_active: Option<bool>,
}

impl PerpDex {
    pub fn is_available(&self) -> bool {
        self.is_active != Some(false) && !self.name.is_empty()
    }
}
