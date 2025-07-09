use async_trait::async_trait;
use bigdecimal::BigDecimal;

use super::client::DeBankClient;
use super::models::DeBankTokenItem;
use crate::error::DeFiError;
use crate::provider::DeFiProvider;
use primitives::{AssetId, Chain, ChainType, DeFiAsset, DeFiPortfolio, DeFiPosition, DeFiPositionFilters, DeFiProtocol, PositionStats};

pub struct DeBankProvider {
    client: DeBankClient,
}

impl DeBankProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            client: DeBankClient::new(api_key),
        }
    }

    fn map_debank_chain(&self, chain_id: &str) -> Option<Chain> {
        match chain_id {
            "eth" => Some(Chain::Ethereum),
            "bsc" => Some(Chain::SmartChain),
            "base" => Some(Chain::Base),
            "arb" => Some(Chain::Arbitrum),
            "matic" => Some(Chain::Polygon),
            "avax" => Some(Chain::AvalancheC),
            "op" => Some(Chain::Optimism),
            "mnt" => Some(Chain::Mantle),
            "ftm" => Some(Chain::Fantom),
            "sonic" => Some(Chain::Sonic),
            "xdai" => Some(Chain::Gnosis),
            "blast" => Some(Chain::Blast),
            "linea" => Some(Chain::Linea),
            "era" => Some(Chain::ZkSync),
            "manta" => Some(Chain::Manta),
            "celo" => Some(Chain::Celo),
            _ => None,
        }
    }

    fn to_debank_chain(&self, chain: &Chain) -> Option<&'static str> {
        match chain {
            Chain::Ethereum => Some("eth"),
            Chain::Polygon => Some("matic"),
            Chain::SmartChain => Some("bsc"),
            Chain::AvalancheC => Some("avax"),
            Chain::Arbitrum => Some("arb"),
            Chain::Optimism => Some("op"),
            Chain::Fantom => Some("ftm"),
            Chain::Base => Some("base"),
            Chain::Linea => Some("linea"),
            Chain::ZkSync => Some("era"),
            Chain::Manta => Some("manta"),
            Chain::Celo => Some("celo"),
            Chain::Mantle => Some("mnt"),
            Chain::Sonic => Some("sonic"),
            Chain::Gnosis => Some("xdai"),
            Chain::Blast => Some("blast"),
            _ => None,
        }
    }

    fn convert_token_to_asset(&self, token: &DeBankTokenItem, asset_type: String) -> Result<DeFiAsset, DeFiError> {
        let chain = self.map_debank_chain(&token.chain).unwrap_or(Chain::Ethereum);
        let asset_id = AssetId {
            chain,
            token_id: Some(token.id.clone()),
        };

        // Calculate USD value from amount and price
        let value_usd = if let Some(price) = &token.price {
            &token.amount * price
        } else {
            BigDecimal::from(0)
        };

        Ok(DeFiAsset {
            asset_id,
            amount: token.amount.clone(),
            value_usd,
            asset_type,
        })
    }
}

#[async_trait]
impl DeFiProvider for DeBankProvider {
    fn name(&self) -> &'static str {
        "DeBank"
    }

    fn supported_chain_types(&self) -> Vec<ChainType> {
        vec![ChainType::Ethereum]
    }

    async fn get_portfolio(&self, address: &str, chains: Vec<Chain>) -> Result<DeFiPortfolio, DeFiError> {
        let mut all_positions = Vec::new();

        let used_chains = if chains.is_empty() {
            // Get used chains if no chains are specified
            self.client
                .get_used_chain_list(address)
                .await?
                .into_iter()
                .filter_map(|debank_chain| self.map_debank_chain(&debank_chain.id))
                .collect()
        } else {
            chains
        };

        let chain_ids = used_chains
            .into_iter()
            .filter_map(|chain| self.to_debank_chain(&chain))
            .collect::<Vec<_>>()
            .join(",");

        let protocol_list = self.client.get_complex_protocol_list(address, &chain_ids).await?;
        for protocol in protocol_list {
            // Skip if chain is not supported
            let chain = self.map_debank_chain(&protocol.chain);
            if chain.is_none() {
                continue;
            }

            if !protocol.portfolio_item_list.is_empty() {
                for item in protocol.portfolio_item_list {
                    let mut assets = Vec::new();

                    // Convert supply tokens
                    if let Some(supply_tokens) = &item.detail.supply_token_list {
                        for token in supply_tokens {
                            if let Ok(asset) = self.convert_token_to_asset(token, "Supply".to_string()) {
                                assets.push(asset);
                            }
                        }
                    }

                    // Convert reward tokens
                    if let Some(reward_tokens) = &item.detail.reward_token_list {
                        for token in reward_tokens {
                            if let Ok(asset) = self.convert_token_to_asset(token, "Reward".to_string()) {
                                assets.push(asset);
                            }
                        }
                    }

                    // Convert borrow tokens
                    if let Some(borrow_tokens) = &item.detail.borrow_token_list {
                        for token in borrow_tokens {
                            if let Ok(asset) = self.convert_token_to_asset(token, "Borrow".to_string()) {
                                assets.push(asset);
                            }
                        }
                    }

                    let position = DeFiPosition {
                        address: address.to_string(),
                        chain: chain.unwrap().to_string(),
                        protocol: DeFiProtocol {
                            id: protocol.id.clone(),
                            name: protocol.name.clone(),
                            logo_url: protocol.logo_url.clone(),
                            website: protocol.site_url.clone(),
                        },
                        position_type: item.detail_types.join(","),
                        name: item.name.clone(),
                        stats: PositionStats {
                            asset_value_usd: item.stats.asset_usd_value.clone(),
                            debt_value_usd: item.stats.debt_usd_value.clone(),
                            net_value_usd: item.stats.net_usd_value.clone(),
                            health_ratio: item.detail.health_rate.clone(),
                        },
                        assets,
                    };

                    all_positions.push(position);
                }
            }
        }

        Ok(DeFiPortfolio { positions: all_positions })
    }

    async fn get_positions(&self, address: &str, filters: Option<DeFiPositionFilters>) -> Result<Vec<DeFiPosition>, DeFiError> {
        let chains = filters.as_ref().map(|f| f.chains.clone()).unwrap_or_default();

        let portfolio = self.get_portfolio(address, chains).await?;
        let mut positions = portfolio.positions;

        if let Some(filters) = filters {
            // Filter by position types
            if !filters.position_types.is_empty() {
                positions.retain(|p| filters.position_types.contains(&p.position_type));
            }

            // Filter by protocols
            if !filters.protocols.is_empty() {
                positions.retain(|p| filters.protocols.contains(&p.protocol.id));
            }
        }

        Ok(positions)
    }
}
