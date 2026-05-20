// Taken from https://github.com/metaplex-foundation/mpl-core/blob/main/programs/mpl-core/src/state/asset.rs
use crate::Pubkey;
use borsh::{BorshDeserialize, BorshSerialize};
use gem_encoding::decode_base64;

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum Key {
    Uninitialized,
    AssetV1,
    HashedAssetV1,
    PluginHeaderV1,
    PluginRegistryV1,
    CollectionV1,
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub enum UpdateAuthority {
    None,
    Address(Pubkey),
    Collection(Pubkey),
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub struct AssetV1 {
    pub key: Key,
    pub owner: Pubkey,
    pub update_authority: UpdateAuthority,
    pub name: String,
    pub uri: String,
    pub seq: Option<u64>,
}

impl AssetV1 {
    pub fn collection(&self) -> Option<Pubkey> {
        match self.update_authority {
            UpdateAuthority::Collection(pubkey) => Some(pubkey),
            UpdateAuthority::Address(_) | UpdateAuthority::None => None,
        }
    }
}

pub fn decode_asset(base64_data: &str) -> Result<AssetV1, Box<dyn std::error::Error + Send + Sync>> {
    let data = decode_base64(base64_data).map_err(|e| format!("decode core asset base64: {e}"))?;
    let asset = AssetV1::deserialize(&mut data.as_slice()).map_err(|e| format!("decode core asset: {e}"))?;
    if asset.key != Key::AssetV1 {
        return Err(format!("unexpected Metaplex Core asset key: {:?}", asset.key).into());
    }
    Ok(asset)
}

#[cfg(test)]
mod tests {
    use super::*;

    const BWED_1545_ASSET: &str = "AdQCoT0whdkMOHS+JayL1er4PFXv7D/dY+IJ+r2UN6wYAkeTzcRWuwoVceHlkHr4rEWE2hX58AjYjrPZJGJKtkYICgAAAEJXRUQgIzE1NDVbAAAAaHR0cHM6Ly9iYWZ5YmVpZm91dzRjdnF6a21pNzN3ZWQzM2lsM3l5cTdnam9xbW91emFvZGI3Z2J4YmNiZm94cXd1cS5pcGZzLnczcy5saW5rLzE1NDUuanNvbgA=";

    #[test]
    fn test_decode_asset() {
        let asset = decode_asset(BWED_1545_ASSET).unwrap();
        assert_eq!(asset.key, Key::AssetV1);
        assert_eq!(asset.owner.to_base58(), "FGbkx8rYTPJubjyScReeps6GA83D1nSmFr3BrN7buokb");
        assert_eq!(asset.collection().unwrap().to_base58(), "5pQfZttNUtaj8sySRY9RsdtB81aEAQDh2vnacpxiwTpT");
        assert_eq!(asset.name, "BWED #1545");
        assert!(asset.uri.contains("1545.json"));
        assert_eq!(asset.seq, None);

        assert!(decode_asset("Y29ycnVwdA==").is_err());
    }
}
