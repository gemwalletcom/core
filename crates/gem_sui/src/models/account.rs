use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Owner {
    String(String),
    OwnerObject(OwnerObject),
}

impl Owner {
    pub fn get_address_owner(&self) -> Option<String> {
        match self {
            Owner::String(_) => None,
            Owner::OwnerObject(obj) => obj.address_owner.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnerObject {
    #[serde(rename = "AddressOwner")]
    pub address_owner: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GasObject {
    pub owner: Owner,
}
