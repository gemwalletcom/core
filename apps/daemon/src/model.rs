use primitives::Chain;
use std::str::FromStr;
use strum::{AsRefStr, EnumIter, EnumString, IntoEnumIterator};

#[derive(Debug, Clone, PartialEq, AsRefStr, EnumString, EnumIter)]
#[strum(serialize_all = "snake_case")]
pub enum ConsumerService {
    FetchAddressTransactions,
    StoreTransactions,
    FetchBlocks,
    FetchAssets,
    FetchTokenAssociations,
    FetchCoinAssociations,
    StoreAssetsAssociations,
    FetchNftAssociations,
    Notifications,
    InAppNotifications,
    Rewards,
    RewardsRedemptions,
    Support,
    Fiat,
    StorePrices,
    StoreCharts,
    FetchPrices,
    Nft,
}

impl ConsumerService {
    pub fn all() -> Vec<Self> {
        Self::iter().collect()
    }
}

#[derive(Debug, Clone, Copy, AsRefStr, EnumString, EnumIter, PartialEq, Eq)]
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
    Scan,
    Rewards,
}

impl WorkerService {
    pub fn all() -> Vec<Self> {
        Self::iter().collect()
    }
}

#[derive(Debug, Clone, AsRefStr)]
#[strum(serialize_all = "snake_case")]
pub enum DaemonService {
    Setup,
    SetupDev,
    #[strum(serialize = "worker")]
    Worker(Option<WorkerService>),
    #[strum(serialize = "parser")]
    Parser(Option<Chain>),
    #[strum(serialize = "consumer")]
    Consumer(Option<ConsumerService>),
}

impl DaemonService {
    pub fn name(&self) -> String {
        match self {
            DaemonService::Setup => "setup".to_owned(),
            DaemonService::SetupDev => "setup_dev".to_owned(),
            DaemonService::Worker(Some(name)) => format!("worker {}", name.as_ref()),
            DaemonService::Worker(None) => "worker all".to_owned(),
            DaemonService::Parser(chain) => {
                if let Some(chain) = chain {
                    format!("parser {}", chain.as_ref())
                } else {
                    "parser".to_owned()
                }
            }
            DaemonService::Consumer(Some(consumer)) => format!("consumer {}", consumer.as_ref()),
            DaemonService::Consumer(None) => "consumer all".to_owned(),
        }
    }
}

impl DaemonService {
    const SETUP: &'static str = "setup";
    const SETUP_DEV: &'static str = "setup_dev";
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
            Self::SETUP_DEV => Ok(DaemonService::SetupDev),
            Self::WORKER => {
                let worker = if let Some(worker_str) = parts.get(1) {
                    Some(WorkerService::from_str(worker_str).map_err(|_| format!("Invalid worker: {}", worker_str))?)
                } else {
                    None
                };
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
                let consumer = if let Some(consumer_str) = parts.get(1) {
                    Some(ConsumerService::from_str(consumer_str).map_err(|_| format!("Invalid consumer: {}", consumer_str))?)
                } else {
                    None
                };
                Ok(DaemonService::Consumer(consumer))
            }
            _ => Err(format!("Unknown service: {}", name)),
        }
    }
}
