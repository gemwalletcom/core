use url::Url;

use crate::AssetId;

const DEEPLINK_HOST: &str = "gemwallet.com";
const DEEPLINK_WEB_SCHEME: &str = "https";
const DEEPLINK_GEM_SCHEME: &str = "gem";

const PATH_TOKENS: &str = "tokens";
const PATH_PERPETUALS: &str = "perpetuals";
const PATH_REWARDS: &str = "rewards";
const PATH_JOIN: &str = "join";

const QUERY_CODE: &str = "code";

#[derive(Debug, Clone, PartialEq)]
pub enum Deeplink {
    Asset { asset_id: AssetId },
    Perpetuals,
    Rewards { code: Option<String> },
}

impl Deeplink {
    pub fn to_url(&self) -> String {
        format!("{DEEPLINK_WEB_SCHEME}://{DEEPLINK_HOST}{}", self.path())
    }

    pub fn to_gem_url(&self) -> String {
        format!("{DEEPLINK_GEM_SCHEME}://{}", self.path().trim_start_matches('/'))
    }

    pub fn from_url(url: &str) -> Option<Self> {
        let url = Url::parse(url).ok()?;
        let segments = url_segments(&url)?;
        let (component, params) = segments.split_first()?;

        let deeplink = match component.as_str() {
            PATH_TOKENS => Deeplink::Asset {
                asset_id: AssetId::from(params.first()?.parse().ok()?, params.get(1).cloned()),
            },
            PATH_PERPETUALS => Deeplink::Perpetuals,
            PATH_REWARDS | PATH_JOIN => Deeplink::Rewards {
                code: params.first().cloned().or_else(|| query_value(&url, QUERY_CODE)),
            },
            _ => return None,
        };
        Some(deeplink)
    }

    fn path(&self) -> String {
        match self {
            Deeplink::Asset { asset_id } => match &asset_id.token_id {
                Some(token_id) => format!("/{PATH_TOKENS}/{}/{token_id}", asset_id.chain.as_ref()),
                None => format!("/{PATH_TOKENS}/{}", asset_id.chain.as_ref()),
            },
            Deeplink::Perpetuals => format!("/{PATH_PERPETUALS}"),
            Deeplink::Rewards { code } => path_with_query(PATH_REWARDS, QUERY_CODE, code.clone()),
        }
    }
}

fn path_with_query(component: &str, query_key: &str, query_value: Option<String>) -> String {
    match query_value {
        Some(value) => format!("/{component}?{query_key}={value}"),
        None => format!("/{component}"),
    }
}

fn url_segments(url: &Url) -> Option<Vec<String>> {
    let mut segments: Vec<String> = url
        .path_segments()
        .map(|parts| parts.filter(|part| !part.is_empty()).map(String::from).collect())
        .unwrap_or_default();

    match url.scheme() {
        DEEPLINK_WEB_SCHEME => {
            if url.host_str()? != DEEPLINK_HOST {
                return None;
            }
        }
        DEEPLINK_GEM_SCHEME => {
            if let Some(host) = url.host_str().filter(|host| !host.is_empty()) {
                segments.insert(0, host.to_string());
            }
        }
        _ => return None,
    }
    Some(segments)
}

fn query_value(url: &Url, key: &str) -> Option<String> {
    url.query_pairs().find(|(query_key, _)| query_key.as_ref() == key).map(|(_, value)| value.into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Chain;

    #[test]
    fn test_to_url() {
        assert_eq!(
            Deeplink::Asset {
                asset_id: AssetId::from_chain(Chain::Bitcoin)
            }
            .to_url(),
            "https://gemwallet.com/tokens/bitcoin"
        );
        assert_eq!(
            Deeplink::Asset {
                asset_id: AssetId::token(Chain::Ethereum, "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"),
            }
            .to_url(),
            "https://gemwallet.com/tokens/ethereum/0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"
        );
        assert_eq!(Deeplink::Perpetuals.to_url(), "https://gemwallet.com/perpetuals");
        assert_eq!(Deeplink::Rewards { code: None }.to_url(), "https://gemwallet.com/rewards");
        assert_eq!(
            Deeplink::Rewards {
                code: Some("gemcoder".to_string()),
            }
            .to_url(),
            "https://gemwallet.com/rewards?code=gemcoder"
        );
    }

    #[test]
    fn test_to_gem_url() {
        assert_eq!(Deeplink::Rewards { code: None }.to_gem_url(), "gem://rewards");
        assert_eq!(Deeplink::Perpetuals.to_gem_url(), "gem://perpetuals");
        assert_eq!(
            Deeplink::Asset {
                asset_id: AssetId::from_chain(Chain::Bitcoin)
            }
            .to_gem_url(),
            "gem://tokens/bitcoin"
        );
    }

    #[test]
    fn test_from_url() {
        assert_eq!(
            Deeplink::from_url("https://gemwallet.com/tokens/bitcoin"),
            Some(Deeplink::Asset {
                asset_id: AssetId::from_chain(Chain::Bitcoin)
            })
        );
        assert_eq!(
            Deeplink::from_url("https://gemwallet.com/tokens/ethereum/0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"),
            Some(Deeplink::Asset {
                asset_id: AssetId::token(Chain::Ethereum, "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"),
            })
        );
        assert_eq!(
            Deeplink::from_url("gem://tokens/bitcoin"),
            Some(Deeplink::Asset {
                asset_id: AssetId::from_chain(Chain::Bitcoin)
            })
        );
        assert_eq!(Deeplink::from_url("https://gemwallet.com/perpetuals"), Some(Deeplink::Perpetuals));
        assert_eq!(Deeplink::from_url("gem://perpetuals"), Some(Deeplink::Perpetuals));
        assert_eq!(
            Deeplink::from_url("https://gemwallet.com/rewards?code=gemcoder"),
            Some(Deeplink::Rewards {
                code: Some("gemcoder".to_string()),
            })
        );
        assert_eq!(
            Deeplink::from_url("https://gemwallet.com/join/gemcoder"),
            Some(Deeplink::Rewards {
                code: Some("gemcoder".to_string()),
            })
        );
        assert_eq!(Deeplink::from_url("https://gemwallet.com/join"), Some(Deeplink::Rewards { code: None }));
        assert_eq!(Deeplink::from_url("https://gemwallet.com/tokens"), None);
        assert_eq!(Deeplink::from_url("https://gemwallet.com/tokens/notachain"), None);
        assert_eq!(Deeplink::from_url("https://example.com/tokens/bitcoin"), None);
        assert_eq!(Deeplink::from_url("https://gemwallet.com/unknown"), None);
        assert_eq!(Deeplink::from_url("not a url"), None);
    }
}
