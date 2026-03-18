use std::collections::HashSet;
use std::error::Error;
use std::sync::Arc;

use cacher::{CacheKey, CacherClient};
use gem_tracing::error_with_fields;
use primitives::{Chain, PerpetualPosition};
use settings_chain::ChainProviders;

#[derive(Clone, Copy)]
pub struct PerpetualPriorityConfig {
    pub trigger_distance: f64,
    pub liquidation_distance: f64,
}

pub struct PerpetualPositionClassifier {
    chain: Chain,
    providers: Arc<ChainProviders>,
    cacher: CacherClient,
    priority_config: PerpetualPriorityConfig,
}

impl PerpetualPositionClassifier {
    pub fn new(chain: Chain, providers: Arc<ChainProviders>, cacher: CacherClient, priority_config: PerpetualPriorityConfig) -> Self {
        Self {
            chain,
            providers,
            cacher,
            priority_config,
        }
    }

    pub async fn classify(&self) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let addresses = self.get_addresses(CacheKey::PerpetualTrackedAddresses(self.chain.as_ref())).await?;
        let current_active = self.get_address_set(CacheKey::PerpetualActiveAddresses(self.chain.as_ref())).await?;
        let current_priority = self.get_address_set(CacheKey::PerpetualPriorityAddresses(self.chain.as_ref())).await?;

        let mut active_addresses = Vec::new();
        let mut priority_addresses = Vec::new();

        for address in &addresses {
            match self.providers.get_perpetual_positions(self.chain, address.clone()).await {
                Ok(summary) => {
                    if !summary.positions.is_empty() {
                        active_addresses.push(address.clone());
                        if summary.positions.iter().any(|p| is_priority_position(p, self.priority_config)) {
                            priority_addresses.push(address.clone());
                        }
                    }
                }
                Err(error) => {
                    if current_active.contains(address.as_str()) {
                        active_addresses.push(address.clone());
                    }
                    if current_priority.contains(address.as_str()) {
                        priority_addresses.push(address.clone());
                    }
                    error_with_fields!("perpetual_classifier", &*error, chain = self.chain.as_ref(), address = address);
                }
            }
        }

        self.cacher.set_cached(CacheKey::PerpetualActiveAddresses(self.chain.as_ref()), &active_addresses).await?;
        self.cacher
            .set_cached(CacheKey::PerpetualPriorityAddresses(self.chain.as_ref()), &priority_addresses)
            .await?;

        Ok(addresses.len())
    }

    async fn get_addresses(&self, key: CacheKey<'_>) -> Result<Vec<String>, Box<dyn Error + Send + Sync>> {
        Ok(self.cacher.get_cached_optional::<Vec<String>>(key).await?.unwrap_or_default())
    }

    async fn get_address_set(&self, key: CacheKey<'_>) -> Result<HashSet<String>, Box<dyn Error + Send + Sync>> {
        Ok(self.get_addresses(key).await?.into_iter().collect())
    }
}

fn is_priority_position(position: &PerpetualPosition, config: PerpetualPriorityConfig) -> bool {
    let Some(mark_price) = current_mark_price(position) else {
        return false;
    };

    let near = |target: f64, threshold: f64| relative_distance(mark_price, target).is_some_and(|d| d <= threshold);

    let near_liquidation = position.liquidation_price.is_some_and(|p| near(p, config.liquidation_distance));
    let near_auto_close = position.take_profit.as_ref().is_some_and(|o| near(o.price, config.trigger_distance))
        || position.stop_loss.as_ref().is_some_and(|o| near(o.price, config.trigger_distance));

    near_liquidation || near_auto_close
}

fn current_mark_price(position: &PerpetualPosition) -> Option<f64> {
    if position.size > 0.0 && position.size_value > 0.0 {
        Some(position.size_value / position.size)
    } else {
        None
    }
}

fn relative_distance(current: f64, target: f64) -> Option<f64> {
    if current <= 0.0 || target <= 0.0 {
        None
    } else {
        Some((target - current).abs() / current)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::{AssetId, Chain, PerpetualDirection, PerpetualMarginType, PerpetualOrderType, PerpetualTriggerOrder};

    fn config() -> PerpetualPriorityConfig {
        PerpetualPriorityConfig {
            trigger_distance: 0.01,
            liquidation_distance: 0.06,
        }
    }

    fn position() -> PerpetualPosition {
        PerpetualPosition {
            id: "1".to_string(),
            perpetual_id: "hypercore_BTC".to_string(),
            asset_id: AssetId::from_token(Chain::HyperCore, "perpetual::BTC"),
            size: 1.0,
            size_value: 100.0,
            leverage: 5,
            entry_price: 100.0,
            liquidation_price: Some(95.0),
            margin_type: PerpetualMarginType::Cross,
            direction: PerpetualDirection::Long,
            margin_amount: 20.0,
            take_profit: None,
            stop_loss: None,
            pnl: 0.0,
            funding: None,
        }
    }

    #[test]
    fn test_near_liquidation() {
        assert!(is_priority_position(&position(), config()));
    }

    #[test]
    fn test_near_auto_close() {
        let mut position = position();
        position.liquidation_price = Some(50.0);
        position.take_profit = Some(PerpetualTriggerOrder {
            price: 100.5,
            order_type: PerpetualOrderType::Limit,
            order_id: "1".to_string(),
        });
        let config = PerpetualPriorityConfig {
            trigger_distance: 0.01,
            liquidation_distance: 0.01,
        };

        assert!(is_priority_position(&position, config));
    }

    #[test]
    fn test_not_priority_when_far() {
        let mut position = position();
        position.liquidation_price = Some(50.0);
        position.stop_loss = Some(PerpetualTriggerOrder {
            price: 80.0,
            order_type: PerpetualOrderType::Market,
            order_id: "2".to_string(),
        });
        let config = PerpetualPriorityConfig {
            trigger_distance: 0.01,
            liquidation_distance: 0.01,
        };

        assert!(!is_priority_position(&position, config));
    }
}
