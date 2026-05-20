use url::Url;

use crate::AssetId;

const DEEPLINK_HOST: &str = "gemwallet.com";
const DEEPLINK_WEB_SCHEME: &str = "https";
const DEEPLINK_GEM_SCHEME: &str = "gem";

const PATH_TOKENS: &str = "tokens";
const PATH_SWAP: &str = "swap";
const PATH_PERPETUALS: &str = "perpetuals";
const PATH_REWARDS: &str = "rewards";
const PATH_JOIN: &str = "join";
const PATH_GIFT: &str = "gift";
const PATH_BUY: &str = "buy";
const PATH_SELL: &str = "sell";
const PATH_SET_PRICE_ALERT: &str = "setPriceAlert";

const QUERY_CODE: &str = "code";
const QUERY_AMOUNT: &str = "amount";
const QUERY_PRICE: &str = "price";

#[derive(Debug, Clone, PartialEq)]
pub enum Deeplink {
    Asset { asset_id: AssetId },
    Swap { from_asset_id: AssetId, to_asset_id: Option<AssetId> },
    Perpetuals,
    Rewards { code: Option<String> },
    Gift { code: Option<String> },
    Buy { asset_id: AssetId, amount: Option<i32> },
    Sell { asset_id: AssetId, amount: Option<i32> },
    SetPriceAlert { asset_id: AssetId, price: Option<f64> },
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
            PATH_SWAP => Deeplink::Swap {
                from_asset_id: AssetId::new(params.first()?)?,
                to_asset_id: params.get(1).and_then(|asset_id| AssetId::new(asset_id)),
            },
            PATH_PERPETUALS => Deeplink::Perpetuals,
            PATH_REWARDS | PATH_JOIN => Deeplink::Rewards {
                code: params.first().cloned().or_else(|| query_value(&url, QUERY_CODE)),
            },
            PATH_GIFT => Deeplink::Gift {
                code: params.first().cloned().or_else(|| query_value(&url, QUERY_CODE)),
            },
            PATH_BUY => Deeplink::Buy {
                asset_id: AssetId::new(params.first()?)?,
                amount: query_value(&url, QUERY_AMOUNT).and_then(|amount| amount.parse().ok()),
            },
            PATH_SELL => Deeplink::Sell {
                asset_id: AssetId::new(params.first()?)?,
                amount: query_value(&url, QUERY_AMOUNT).and_then(|amount| amount.parse().ok()),
            },
            PATH_SET_PRICE_ALERT => Deeplink::SetPriceAlert {
                asset_id: AssetId::new(params.first()?)?,
                price: query_value(&url, QUERY_PRICE).and_then(|price| price.parse().ok()),
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
            Deeplink::Swap { from_asset_id, to_asset_id } => match to_asset_id {
                Some(to_asset_id) => format!("/{PATH_SWAP}/{from_asset_id}/{to_asset_id}"),
                None => format!("/{PATH_SWAP}/{from_asset_id}"),
            },
            Deeplink::Perpetuals => format!("/{PATH_PERPETUALS}"),
            Deeplink::Rewards { code } => path_with_query(PATH_REWARDS, None, QUERY_CODE, code.clone()),
            Deeplink::Gift { code } => path_with_query(PATH_GIFT, None, QUERY_CODE, code.clone()),
            Deeplink::Buy { asset_id, amount } => path_with_query(PATH_BUY, Some(asset_id), QUERY_AMOUNT, amount.as_ref().map(|amount| amount.to_string())),
            Deeplink::Sell { asset_id, amount } => path_with_query(PATH_SELL, Some(asset_id), QUERY_AMOUNT, amount.as_ref().map(|amount| amount.to_string())),
            Deeplink::SetPriceAlert { asset_id, price } => path_with_query(PATH_SET_PRICE_ALERT, Some(asset_id), QUERY_PRICE, price.as_ref().map(|price| price.to_string())),
        }
    }
}

fn path_with_query(component: &str, asset_id: Option<&AssetId>, query_key: &str, query_value: Option<String>) -> String {
    let path = match asset_id {
        Some(asset_id) => format!("/{component}/{asset_id}"),
        None => format!("/{component}"),
    };
    match query_value {
        Some(value) => format!("{path}?{query_key}={value}"),
        None => path,
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
        assert_eq!(
            Deeplink::Swap {
                from_asset_id: AssetId::from_chain(Chain::Ethereum),
                to_asset_id: Some(AssetId::from_chain(Chain::Bitcoin)),
            }
            .to_url(),
            "https://gemwallet.com/swap/ethereum/bitcoin"
        );
        assert_eq!(
            Deeplink::Swap {
                from_asset_id: AssetId::from_chain(Chain::Ethereum),
                to_asset_id: None,
            }
            .to_url(),
            "https://gemwallet.com/swap/ethereum"
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
        assert_eq!(Deeplink::Gift { code: Some("abc".to_string()) }.to_url(), "https://gemwallet.com/gift?code=abc");
        assert_eq!(
            Deeplink::Buy {
                asset_id: AssetId::from_chain(Chain::Ethereum),
                amount: Some(100),
            }
            .to_url(),
            "https://gemwallet.com/buy/ethereum?amount=100"
        );
        assert_eq!(
            Deeplink::Sell {
                asset_id: AssetId::from_chain(Chain::Ethereum),
                amount: None,
            }
            .to_url(),
            "https://gemwallet.com/sell/ethereum"
        );
        assert_eq!(
            Deeplink::SetPriceAlert {
                asset_id: AssetId::from_chain(Chain::Ethereum),
                price: Some(2.5),
            }
            .to_url(),
            "https://gemwallet.com/setPriceAlert/ethereum?price=2.5"
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
        assert_eq!(
            Deeplink::from_url("https://gemwallet.com/swap/ethereum/bitcoin"),
            Some(Deeplink::Swap {
                from_asset_id: AssetId::from_chain(Chain::Ethereum),
                to_asset_id: Some(AssetId::from_chain(Chain::Bitcoin)),
            })
        );
        assert_eq!(
            Deeplink::from_url("https://gemwallet.com/swap/ethereum"),
            Some(Deeplink::Swap {
                from_asset_id: AssetId::from_chain(Chain::Ethereum),
                to_asset_id: None,
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
        assert_eq!(
            Deeplink::from_url("https://gemwallet.com/gift?code=abc"),
            Some(Deeplink::Gift { code: Some("abc".to_string()) })
        );
        assert_eq!(
            Deeplink::from_url("https://gemwallet.com/buy/ethereum?amount=100"),
            Some(Deeplink::Buy {
                asset_id: AssetId::from_chain(Chain::Ethereum),
                amount: Some(100),
            })
        );
        assert_eq!(
            Deeplink::from_url("https://gemwallet.com/sell/ethereum"),
            Some(Deeplink::Sell {
                asset_id: AssetId::from_chain(Chain::Ethereum),
                amount: None,
            })
        );
        assert_eq!(
            Deeplink::from_url("https://gemwallet.com/setPriceAlert/ethereum?price=2.5"),
            Some(Deeplink::SetPriceAlert {
                asset_id: AssetId::from_chain(Chain::Ethereum),
                price: Some(2.5),
            })
        );
        assert_eq!(
            Deeplink::from_url("https://gemwallet.com/setPriceAlert/ethereum"),
            Some(Deeplink::SetPriceAlert {
                asset_id: AssetId::from_chain(Chain::Ethereum),
                price: None,
            })
        );
        assert_eq!(Deeplink::from_url("https://gemwallet.com/tokens"), None);
        assert_eq!(Deeplink::from_url("https://gemwallet.com/tokens/notachain"), None);
        assert_eq!(Deeplink::from_url("https://example.com/tokens/bitcoin"), None);
        assert_eq!(Deeplink::from_url("https://gemwallet.com/unknown"), None);
        assert_eq!(Deeplink::from_url("not a url"), None);
    }
}
