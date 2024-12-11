use gem_solana::{
    metaplex::{decode_metadata, metadata::Metadata},
    pubkey::Pubkey,
};
use std::str::FromStr;

#[derive(uniffi::Record, Clone)]
pub struct MplMetadata {
    pub mint: String,
    pub name: String,
    pub symbol: String,
    pub uri: String,
}

pub fn decode_mpl_metadata(base64_str: String) -> Result<MplMetadata, Box<dyn std::error::Error>> {
    let metadata = decode_metadata(&base64_str)?;
    Ok(MplMetadata {
        mint: metadata.mint.to_string(),
        name: metadata.data.name.trim_matches(char::from(0)).into(),
        symbol: metadata.data.symbol.trim_matches(char::from(0)).into(),
        uri: metadata.data.uri.trim_matches(char::from(0)).into(),
    })
}

pub fn derive_metadata_pda(mint: &str) -> Result<String, Box<dyn std::error::Error>> {
    let key = Pubkey::from_str(mint)?;
    let metadata = Metadata::find_pda(key).ok_or("metadata program account not found")?;
    Ok(metadata.0.to_string())
}
