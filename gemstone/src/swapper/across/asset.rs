use primitives::{Asset, AssetId};

use super::chain::AcrossChainName;

#[derive(Clone)]
pub struct AcrossChainAsset {
    pub symbol: String,
    pub chain: AcrossChainName,
    pub token_id: Option<String>,
    pub decimals: u32,
}

impl AcrossChainAsset {
    pub fn asset_name(&self) -> String {
        if self.token_id.is_some() {
            format!("{}.{}", self.chain.long_name(), self.symbol)
        } else {
            self.chain.short_name().to_string()
        }
    }

    pub fn from_asset_id(asset_id: AssetId) -> Option<AcrossChainAsset> {
        let chain = AcrossChainName::from_chain(&asset_id.chain)?;
        if let Some(token_id) = &asset_id.token_id {
            AcrossChainAsset::from(chain, token_id)
        } else {
            let asset = Asset::from_chain(asset_id.chain);
            Some(AcrossChainAsset {
                symbol: asset.symbol,
                chain,
                token_id: None,
                decimals: asset.decimals as u32,
            })
        }
    }

    pub fn from(chain: AcrossChainName, token_id: &str) -> Option<AcrossChainAsset> {
        match chain {
            AcrossChainName::Ethereum => match token_id {
                "0xdAC17F958D2ee523a2206206994597C13D831ec7" => Some(AcrossChainAsset {
                    symbol: "USDT".to_string(),
                    chain: chain.clone(),
                    token_id: Some(token_id.to_owned()),
                    decimals: 6,
                }),
                "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48" => Some(AcrossChainAsset {
                    symbol: "USDC".to_string(),
                    chain: chain.clone(),
                    token_id: Some(token_id.to_owned()),
                    decimals: 6,
                }),
                "0x2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599" => Some(AcrossChainAsset {
                    symbol: "WBTC".to_string(),
                    chain: chain.clone(),
                    token_id: Some(token_id.to_owned()),
                    decimals: 8,
                }),

                "0x04Fa0d235C4abf4BcF4787aF4CF447DE572eF828" => Some(AcrossChainAsset {
                    symbol: "UMA".to_string(),
                    chain: chain.clone(),
                    token_id: Some(token_id.to_owned()),
                    decimals: 18,
                }),
                "0xba100000625a3754423978a60c9317c58a424e3D" => Some(AcrossChainAsset {
                    symbol: "BAL".to_string(),
                    chain: chain.clone(),
                    token_id: Some(token_id.to_owned()),
                    decimals: 18,
                }),
                "0x44108f0223A3C3028F5Fe7AEC7f9bb2E66beF82F" => Some(AcrossChainAsset {
                    symbol: "ACX".to_string(),
                    chain: chain.clone(),
                    token_id: Some(token_id.to_owned()),
                    decimals: 18,
                }),
                "0xC011a73ee8576Fb46F5E1c5751cA3B9Fe0af2a6F" => Some(AcrossChainAsset {
                    symbol: "SNX".to_string(),
                    chain: chain.clone(),
                    token_id: Some(token_id.to_owned()),
                    decimals: 18,
                }),
                "0x0cEC1A9154Ff802e7934Fc916Ed7Ca50bDE6844e" => Some(AcrossChainAsset {
                    symbol: "POOL".to_string(),
                    chain: chain.clone(),
                    token_id: Some(token_id.to_owned()),
                    decimals: 18,
                }),
                "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2" => Some(AcrossChainAsset {
                    symbol: "ETH".to_string(),
                    chain: chain.clone(),
                    token_id: Some(token_id.to_owned()),
                    decimals: 18,
                }),
                "0x6B175474E89094C44Da98b954EedeAC495271d0F" => Some(AcrossChainAsset {
                    symbol: "DAI".to_string(),
                    chain: chain.clone(),
                    token_id: Some(token_id.to_owned()),
                    decimals: 18,
                }),
                _ => None,
            },
            AcrossChainName::Optimism => match token_id {
                "0x4200000000000000000000000000000000000006" => Some(AcrossChainAsset {
                    symbol: "ETH".to_string(),
                    chain: chain.clone(),
                    token_id: Some(token_id.to_owned()),
                    decimals: 18,
                }),
                "0x0b2C639c533813f4Aa9D7837CAf62653d097Ff85" => Some(AcrossChainAsset {
                    symbol: "USDC".to_string(),
                    chain: chain.clone(),
                    token_id: Some(token_id.to_owned()),
                    decimals: 18,
                }),
                "0x68f180fcCe6836688e9084f035309E29Bf0A2095" => Some(AcrossChainAsset {
                    symbol: "WBTC".to_string(),
                    chain: chain.clone(),
                    token_id: Some(token_id.to_owned()),
                    decimals: 18,
                }),
                "0xE7798f023fC62146e8Aa1b36Da45fb70855a77Ea" => Some(AcrossChainAsset {
                    symbol: "UMA".to_string(),
                    chain: chain.clone(),
                    token_id: Some(token_id.to_owned()),
                    decimals: 18,
                }),
                "0xDA10009cBd5D07dd0CeCc66161FC93D7c9000da1" => Some(AcrossChainAsset {
                    symbol: "DAI".to_string(),
                    chain: chain.clone(),
                    token_id: Some(token_id.to_owned()),
                    decimals: 18,
                }),
                "0xFE8B128bA8C78aabC59d4c64cEE7fF28e9379921" => Some(AcrossChainAsset {
                    symbol: "BAL".to_string(),
                    chain: chain.clone(),
                    token_id: Some(token_id.to_owned()),
                    decimals: 18,
                }),
                "0xFf733b2A3557a7ed6697007ab5D11B79FdD1b76B" => Some(AcrossChainAsset {
                    symbol: "ACX".to_string(),
                    chain: chain.clone(),
                    token_id: Some(token_id.to_owned()),
                    decimals: 18,
                }),
                "0x94b008aA00579c1307B0EF2c499aD98a8ce58e58" => Some(AcrossChainAsset {
                    symbol: "USDT".to_string(),
                    chain: chain.clone(),
                    token_id: Some(token_id.to_owned()),
                    decimals: 18,
                }),
                "0x8700dAec35aF8Ff88c16BdF0418774CB3D7599B4" => Some(AcrossChainAsset {
                    symbol: "SNX".to_string(),
                    chain: chain.clone(),
                    token_id: Some(token_id.to_owned()),
                    decimals: 18,
                }),
                "0x395Ae52bB17aef68C2888d941736A71dC6d4e125" => Some(AcrossChainAsset {
                    symbol: "POOL".to_string(),
                    chain: chain.clone(),
                    token_id: Some(token_id.to_owned()),
                    decimals: 18,
                }),
                _ => None,
            },
            AcrossChainName::Polygon => match token_id {
                "0x8f3Cf7ad23Cd3CaDbD9735AFf958023239c6A063" => Some(AcrossChainAsset {
                    symbol: "DAI".to_string(),
                    chain: chain.clone(),
                    token_id: Some(token_id.to_owned()),
                    decimals: 18,
                }),
                "0x3066818837c5e6eD6601bd5a91B0762877A6B731" => Some(AcrossChainAsset {
                    symbol: "UMA".to_string(),
                    chain: chain.clone(),
                    token_id: Some(token_id.to_owned()),
                    decimals: 18,
                }),
                "0x3c499c542cEF5E3811e1192ce70d8cC03d5c3359" => Some(AcrossChainAsset {
                    symbol: "USDC".to_string(),
                    chain: chain.clone(),
                    token_id: Some(token_id.to_owned()),
                    decimals: 18,
                }),
                "0x1BFD67037B42Cf73acF2047067bd4F2C47D9BfD6" => Some(AcrossChainAsset {
                    symbol: "WBTC".to_string(),
                    chain: chain.clone(),
                    token_id: Some(token_id.to_owned()),
                    decimals: 18,
                }),
                "0x9a71012B13CA4d3D0Cdc72A177DF3ef03b0E76A3" => Some(AcrossChainAsset {
                    symbol: "BAL".to_string(),
                    chain: chain.clone(),
                    token_id: Some(token_id.to_owned()),
                    decimals: 18,
                }),
                "0xF328b73B6c685831F238c30a23Fc19140CB4D8FC" => Some(AcrossChainAsset {
                    symbol: "ACX".to_string(),
                    chain: chain.clone(),
                    token_id: Some(token_id.to_owned()),
                    decimals: 18,
                }),
                "0xc2132D05D31c914a87C6611C10748AEb04B58e8F" => Some(AcrossChainAsset {
                    symbol: "USDT".to_string(),
                    chain: chain.clone(),
                    token_id: Some(token_id.to_owned()),
                    decimals: 18,
                }),
                "0x25788a1a171ec66Da6502f9975a15B609fF54CF6" => Some(AcrossChainAsset {
                    symbol: "POOL".to_string(),
                    chain: chain.clone(),
                    token_id: Some(token_id.to_owned()),
                    decimals: 18,
                }),
                "0x7ceB23fD6bC0adD59E62ac25578270cFf1b9f619" => Some(AcrossChainAsset {
                    symbol: "ETH".to_string(),
                    chain: chain.clone(),
                    token_id: Some(token_id.to_owned()),
                    decimals: 18,
                }),
                _ => None,
            },
            AcrossChainName::Arbitrum => match token_id {
                "0xDA10009cBd5D07dd0CeCc66161FC93D7c9000da1" => Some(AcrossChainAsset {
                    symbol: "DAI".to_string(),
                    chain: chain.clone(),
                    token_id: Some(token_id.to_owned()),
                    decimals: 18,
                }),
                "0xd693Ec944A85eeca4247eC1c3b130DCa9B0C3b22" => Some(AcrossChainAsset {
                    symbol: "UMA".to_string(),
                    chain: chain.clone(),
                    token_id: Some(token_id.to_owned()),
                    decimals: 18,
                }),
                "0x82aF49447D8a07e3bd95BD0d56f35241523fBab1" => Some(AcrossChainAsset {
                    symbol: "WETH".to_string(),
                    chain: chain.clone(),
                    token_id: Some(token_id.to_owned()),
                    decimals: 18,
                }),
                "0xaf88d065e77c8cC2239327C5EDb3A432268e5831" => Some(AcrossChainAsset {
                    symbol: "USDC".to_string(),
                    chain: chain.clone(),
                    token_id: Some(token_id.to_owned()),
                    decimals: 18,
                }),
                "0x2f2a2543B76A4166549F7aaB2e75Bef0aefC5B0f" => Some(AcrossChainAsset {
                    symbol: "WBTC".to_string(),
                    chain: chain.clone(),
                    token_id: Some(token_id.to_owned()),
                    decimals: 18,
                }),
                "0x040d1EdC9569d4Bab2D15287Dc5A4F10F56a56B8" => Some(AcrossChainAsset {
                    symbol: "BAL".to_string(),
                    chain: chain.clone(),
                    token_id: Some(token_id.to_owned()),
                    decimals: 18,
                }),
                "0x53691596d1BCe8CEa565b84d4915e69e03d9C99d" => Some(AcrossChainAsset {
                    symbol: "ACX".to_string(),
                    chain: chain.clone(),
                    token_id: Some(token_id.to_owned()),
                    decimals: 18,
                }),
                "0xFd086bC7CD5C481DCC9C85ebE478A1C0b69FCbb9" => Some(AcrossChainAsset {
                    symbol: "USDT".to_string(),
                    chain: chain.clone(),
                    token_id: Some(token_id.to_owned()),
                    decimals: 18,
                }),
                "0xCF934E2402A5e072928a39a956964eb8F2B5B79C" => Some(AcrossChainAsset {
                    symbol: "POOL".to_string(),
                    chain: chain.clone(),
                    token_id: Some(token_id.to_owned()),
                    decimals: 18,
                }),
                _ => None,
            },
            AcrossChainName::Base => match token_id {
                "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913" => Some(AcrossChainAsset {
                    symbol: "USDC".to_string(),
                    chain: chain.clone(),
                    token_id: Some(token_id.to_owned()),
                    decimals: 18,
                }),
                "0x4200000000000000000000000000000000000006" => Some(AcrossChainAsset {
                    symbol: "ETH".to_string(),
                    chain: chain.clone(),
                    token_id: Some(token_id.to_owned()),
                    decimals: 18,
                }),
                _ => None,
            },

            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::Chain;
}
