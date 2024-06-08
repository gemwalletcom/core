use crate::AssetId;

pub struct ImageFormatter {}

impl ImageFormatter {
    pub fn get_asset_url(url: &str, asset_id: AssetId) -> String {
        match asset_id.token_id {
            Some(token_id) => format!(
                "{}/blockchains/{}/assets/{}/logo.png",
                url, asset_id.chain, token_id
            ),
            None => format!("{}/blockchains/{}/logo.png", url, asset_id.chain),
        }
    }

    pub fn get_validator_url(url: &str, chain: &str, id: &str) -> String {
        format!("{}/blockchains/{}/validators/{}/logo.png", url, chain, id)
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
            ImageFormatter::get_asset_url(URL, AssetId::from_chain(Chain::Ethereum)),
            "https://example.com/blockchains/ethereum/logo.png"
        );

        assert_eq!(
            ImageFormatter::get_asset_url(
                URL,
                AssetId::from(Chain::Ethereum, Some(String::from("1")))
            ),
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
