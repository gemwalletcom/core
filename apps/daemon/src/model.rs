use primitives::Chain;
use std::str::FromStr;
use strum::{AsRefStr, EnumString};

#[derive(Debug, Clone, AsRefStr, EnumString, PartialEq)]
#[strum(serialize_all = "snake_case")]
pub enum ConsumerService {
    FetchTransactions,
    StoreTransactions,
    FetchBlocks,
    FetchAssets,
    FetchTokenAddressesMappings,
    FetchCoinAddressesMappings,
    StoreAssetsMappings,
    FetchNftAssetsMappings,
    Notifications,
    Support,
    Fiat,
}

#[derive(Debug, Clone, AsRefStr, EnumString, PartialEq)]
#[strum(serialize_all = "snake_case")]
pub enum WorkerService {
    Alerter,
    Pricer,
    PricesDex,
    Fiat,
    Assets,
    Version,
    Transaction,
    Device,
    Search,
    Nft,
    Scan,
}

#[derive(Debug, Clone, AsRefStr)]
#[strum(serialize_all = "snake_case")]
pub enum DaemonService {
    Setup,
    #[strum(serialize = "worker")]
    Worker(WorkerService),
    #[strum(serialize = "parser")]
    Parser(Option<Chain>),
    #[strum(serialize = "consumer")]
    Consumer(ConsumerService),
}

impl DaemonService {
    const SETUP: &'static str = "setup";
    const WORKER: &'static str = "worker";
    const PARSER: &'static str = "parser";
    const CONSUMER: &'static str = "consumer";
}

impl FromStr for DaemonService {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split_whitespace().collect();
        let name = parts.first().copied().ok_or_else(|| "Empty service name".to_string())?;

        match name {
            Self::SETUP => Ok(DaemonService::Setup),
            Self::WORKER => {
                let worker_str = parts.get(1).ok_or_else(|| "Worker service must be specified".to_string())?;
                let worker = WorkerService::from_str(worker_str).map_err(|_| format!("Invalid worker: {}", worker_str))?;
                Ok(DaemonService::Worker(worker))
            }
            Self::PARSER => {
                let chain = if let Some(chain_str) = parts.get(1) {
                    Some(Chain::from_str(chain_str).map_err(|_| format!("Invalid chain: {}", chain_str))?)
                } else {
                    None
                };
                Ok(DaemonService::Parser(chain))
            }
            Self::CONSUMER => {
                let consumer_str = parts.get(1).ok_or_else(|| "Consumer service must be specified".to_string())?;
                let consumer = ConsumerService::from_str(consumer_str).map_err(|_| format!("Invalid consumer: {}", consumer_str))?;
                Ok(DaemonService::Consumer(consumer))
            }
            _ => Err(format!("Unknown service: {}", name)),
        }
    }
}
