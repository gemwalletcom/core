use gem_solana::metaplex::decode_metadata;
#[derive(uniffi::Record, Clone)]
pub struct MplMetadata {
    pub mint: String,
    pub name: String,
    pub symbol: String,
    pub uri: String,
}

pub fn decode_mpl_metadata(base64_str: String) -> Result<MplMetadata, Box<dyn std::error::Error>> {
    let metadata = decode_metadata(base64_str)?;
    Ok(MplMetadata {
        mint: metadata.mint.to_string(),
        name: metadata.data.name.trim_matches(char::from(0)).into(),
        symbol: metadata.data.symbol.trim_matches(char::from(0)).into(),
        uri: metadata.data.uri.trim_matches(char::from(0)).into(),
    })
}
