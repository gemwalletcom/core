use serde::Deserialize;

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Nft {
    pub mint_address: String,
    pub name: String,
    pub image: String,
    pub collection: String,
    pub attributes: Vec<Trait>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Trait {
    pub trait_type: String,
    pub value: serde_json::Value,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Collection {
    pub symbol: String,
    pub name: String,
    pub description: String,
    pub image: String,
    pub twitter: Option<String>,
    pub discord: Option<String>,
    pub website: Option<String>,
}
