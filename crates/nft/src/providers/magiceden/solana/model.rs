use serde::Deserialize;

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Nft {
    pub mint_address: String,
    pub owner: String,
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
    pub symbol: Option<String>,
    pub name: String,
    pub description: String,
    pub image: String,
    pub on_chain_collection_address: String,
    pub twitter: Option<String>,
    pub discord: Option<String>,
    pub website: Option<String>,
}
