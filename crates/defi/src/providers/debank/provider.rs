use async_trait::async_trait;
use bigdecimal::BigDecimal;
use std::collections::HashMap;
use uuid::Uuid;

use super::client::DeBankClient;
use super::models::{DeBankMapping, DeBankTokenItem};
use crate::error::DeFiError;
use crate::provider::DeFiProvider;
use primitives::{
    AssetId, Chain, DeFiAsset, DeFiAssetType, DeFiPortfolio, DeFiPosition, DeFiPositionFilters, DeFiPositionType, DeFiProtocol, DeFiProtocolCategory,
    PortfolioSummary, PositionMetadata, PositionStats,
};

pub struct DeBankProvider {
    client: DeBankClient,
}

impl DeBankProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            client: DeBankClient::new(api_key),
        }
    }

    fn convert_token_to_asset(&self, token: &DeBankTokenItem, asset_type: DeFiAssetType) -> Result<DeFiAsset, DeFiError> {
        let chain = DeBankMapping::debank_id_to_chain(&token.chain).unwrap_or(Chain::Ethereum);
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
            yield_info: None,
            attributes: None,
        })
    }

    fn convert_protocol(&self, protocol: &crate::providers::debank::models::DeBankProtocol) -> DeFiProtocol {
        let category = match protocol.name.to_lowercase().as_str() {
            name if name.contains("uniswap") || name.contains("pancake") || name.contains("sushi") => DeFiProtocolCategory::Dex,
            name if name.contains("aave") || name.contains("compound") || name.contains("maker") => DeFiProtocolCategory::Lending,
            name if name.contains("lido") || name.contains("rocket") => DeFiProtocolCategory::Staking,
            name if name.contains("yearn") || name.contains("beefy") => DeFiProtocolCategory::Yield,
            _ => DeFiProtocolCategory::Wallet,
        };

        DeFiProtocol {
            id: protocol.id.clone(),
            name: protocol.name.clone(),
            category,
            logo_url: protocol.logo_url.clone(),
            website: protocol.site_url.clone(),
        }
    }

    fn convert_position_type(&self, detail_types: &[String]) -> DeFiPositionType {
        if let Some(detail_type) = detail_types.iter().next() {
            return DeBankMapping::position_type_from_detail_type(detail_type);
        }
        DeFiPositionType::Wallet
    }
}

#[async_trait]
impl DeFiProvider for DeBankProvider {
    fn name(&self) -> &'static str {
        "DeBank"
    }

    fn supported_chains(&self) -> Vec<Chain> {
        vec![
            Chain::Ethereum,
            Chain::Polygon,
            Chain::SmartChain,
            Chain::AvalancheC,
            Chain::Arbitrum,
            Chain::Optimism,
            Chain::Fantom,
            Chain::Base,
        ]
    }

    async fn get_portfolio(&self, address: &str, chains: Vec<Chain>) -> Result<DeFiPortfolio, DeFiError> {
        let mut all_positions = Vec::new();
        let mut value_by_protocol = HashMap::new();
        let total_value = BigDecimal::from(0);

        let used_chains = if chains.is_empty() {
            // Get used chains from DeBank API
            self.client
                .get_used_chain_list(address)
                .await?
                .into_iter()
                .filter_map(|debank_chain| DeBankMapping::debank_id_to_chain(&debank_chain.id))
                .collect()
        } else {
            chains
        };

        let chain_ids = used_chains
            .into_iter()
            .filter_map(|chain| DeBankMapping::chain_to_debank_id(&chain))
            .collect::<Vec<_>>();

        let chain_ids_str = chain_ids.join(",");

        let protocol_list = self.client.get_complex_protocol_list(address, &chain_ids_str).await?;
        for protocol in protocol_list {
            if let Some(ref portfolio_items) = protocol.portfolio_item_list {
                for item in portfolio_items {
                    let mut assets = Vec::new();

                    // Convert supply tokens
                    if let Some(supply_tokens) = &item.detail.supply_token_list {
                        for token in supply_tokens {
                            if let Ok(asset) = self.convert_token_to_asset(token, DeFiAssetType::Supply) {
                                assets.push(asset);
                            }
                        }
                    }

                    // Convert reward tokens
                    if let Some(reward_tokens) = &item.detail.reward_token_list {
                        for token in reward_tokens {
                            if let Ok(asset) = self.convert_token_to_asset(token, DeFiAssetType::Reward) {
                                assets.push(asset);
                            }
                        }
                    }

                    // Convert borrow tokens
                    if let Some(borrow_tokens) = &item.detail.borrow_token_list {
                        for token in borrow_tokens {
                            if let Ok(asset) = self.convert_token_to_asset(token, DeFiAssetType::Borrow) {
                                assets.push(asset);
                            }
                        }
                    }

                    let position = DeFiPosition {
                        id: Uuid::new_v4().to_string(),
                        address: address.to_string(),
                        chain_id: protocol.chain.clone(),
                        protocol: self.convert_protocol(&protocol),
                        position_type: self.convert_position_type(&item.detail_types),
                        name: item.name.clone(),
                        stats: PositionStats {
                            total_value_usd: &item.stats.asset_usd_value + &item.stats.debt_usd_value,
                            asset_value_usd: item.stats.asset_usd_value.clone(),
                            debt_value_usd: item.stats.debt_usd_value.clone(),
                            net_value_usd: item.stats.net_usd_value.clone(),
                            rewards_value_usd: None,
                            daily_yield_usd: None,
                            apy: None,
                            health_ratio: item.detail.health_rate.clone(),
                            updated_at: None,
                        },
                        assets,
                        metadata: PositionMetadata {
                            created_at: None,
                            last_interaction_at: None,
                            last_tx_hash: None,
                            protocol_position_id: item.pool.as_ref().map(|p| p.id.clone()),
                            extra: None,
                        },
                    };

                    value_by_protocol
                        .entry(protocol.name.clone())
                        .and_modify(|v: &mut BigDecimal| *v += &position.stats.net_value_usd)
                        .or_insert(position.stats.net_value_usd.clone());

                    all_positions.push(position);
                }
            }
        }

        Ok(DeFiPortfolio {
            address: address.to_string(),
            chain_id: "multi".to_string(),
            summary: PortfolioSummary {
                total_value_usd: total_value,
                value_by_protocol,
                total_yield_usd: None,
                health_score: None,
                performance: None,
            },
            positions: all_positions,
        })
    }

    async fn get_positions(&self, address: &str, filters: Option<DeFiPositionFilters>) -> Result<Vec<DeFiPosition>, DeFiError> {
        let chains = filters.as_ref().map(|f| f.chains.clone()).unwrap_or_default();

        let portfolio = self.get_portfolio(address, chains).await?;
        let mut positions = portfolio.positions;

        // Apply filters
        if let Some(filters) = filters {
            // Filter by position types
            if let Some(ref types) = filters.position_types {
                positions.retain(|p| types.contains(&p.position_type));
            }

            // Filter by protocols
            if let Some(ref protocols) = filters.protocols {
                positions.retain(|p| protocols.contains(&p.protocol.id));
            }

            // Filter by debt
            if let Some(has_debt) = filters.has_debt {
                positions.retain(|p| {
                    let has_debt_position = p.stats.debt_value_usd > BigDecimal::from(0);
                    has_debt_position == has_debt
                });
            }

            // Filter by rewards
            if let Some(has_rewards) = filters.has_rewards {
                positions.retain(|p| {
                    let has_reward_assets = p.assets.iter().any(|a| matches!(a.asset_type, DeFiAssetType::Reward));
                    has_reward_assets == has_rewards
                });
            }
        }

        Ok(positions)
    }
}
