use crate::AssetId;

pub struct ImageFormatter {}

impl ImageFormatter {
    pub fn get_asset_url(url: &str, chain: &str, token_id: Option<&str>) -> String {
        match token_id {
            Some(token_id) => format!("{url}/blockchains/{chain}/assets/{token_id}/logo.png"),
            None => format!("{url}/blockchains/{chain}/logo.png"),
        }
    }

    pub fn get_asset_url_for_asset_id(url: &str, asset_id: AssetId) -> String {
        Self::get_asset_url(url, asset_id.chain.as_ref(), asset_id.token_id.as_deref())
    }

    pub fn get_validator_url(url: &str, chain: &str, id: &str) -> String {
        format!("{url}/blockchains/{chain}/validators/{id}/logo.png")
    }

    pub fn get_nft_asset_url(url: &str, id: &str) -> String {
        format!("{url}/v1/nft/assets/{id}/image_preview")
    }
}
#[cfg(test)]
mod tests {
    const URL: &str = "https://example.com";

    use crate::Chain;

    use super::*;

    #[test]
    fn test_get_asset_url() {
        assert_eq!(
            ImageFormatter::get_asset_url_for_asset_id(URL, AssetId::from_chain(Chain::Ethereum)),
            "https://example.com/blockchains/ethereum/logo.png"
        );

        assert_eq!(
            ImageFormatter::get_asset_url_for_asset_id(URL, AssetId::from(Chain::Ethereum, Some(String::from("1")))),
            "https://example.com/blockchains/ethereum/assets/1/logo.png"
        );
    }

    #[test]
    fn test_get_validator_url() {
        assert_eq!(
            ImageFormatter::get_validator_url(URL, Chain::Ethereum.as_ref(), "1"),
            "https://example.com/blockchains/ethereum/validators/1/logo.png"
        );
    }
}
