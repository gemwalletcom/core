use primitives::{Chain, PriceProvider};
use std::str::FromStr;
use strum::{AsRefStr, EnumIter, EnumString, IntoEnumIterator};

#[derive(Debug, Clone, PartialEq, AsRefStr, EnumString, EnumIter)]
#[strum(serialize_all = "snake_case")]
pub enum ConsumerService {
    Store,
    Indexer,
    Notifications,
    Rewards,
    Support,
    Fiat,
}

impl ConsumerService {
    pub fn all() -> Vec<Self> {
        Self::iter().collect()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, AsRefStr, EnumString)]
#[strum(serialize_all = "snake_case")]
#[allow(clippy::enum_variant_names)]
pub enum IndexerConsumer {
    FetchAssets,
    FetchPrices,
    FetchBlocks,
    FetchTokenAssociations,
    FetchCoinAssociations,
    FetchNftAssociations,
    FetchAddressTransactions,
}

#[derive(Debug, Clone)]
pub struct ConsumerOptions {
    pub service: Option<ConsumerService>,
    pub indexer: Option<IndexerConsumer>,
}

#[derive(Debug, Clone, Copy, AsRefStr, EnumString, EnumIter, PartialEq, Eq)]
#[strum(serialize_all = "snake_case")]
pub enum WorkerService {
    Alerter,
    Prices,
    Fiat,
    Assets,
    System,
    Search,
    Rewards,
    Transactions,
    Perpetuals,
}

impl WorkerService {
    pub fn all() -> Vec<Self> {
        Self::iter().collect()
    }
}

#[derive(Debug, Clone)]
pub struct WorkerOptions {
    pub service: Option<WorkerService>,
    pub price_provider: Option<PriceProvider>,
}

#[derive(Debug, Clone, AsRefStr)]
#[strum(serialize_all = "snake_case")]
pub enum DaemonService {
    Setup,
    SetupDev,
    #[strum(serialize = "worker")]
    Worker(WorkerOptions),
    #[strum(serialize = "parser")]
    Parser(Option<Chain>),
    #[strum(serialize = "consumer")]
    Consumer(ConsumerOptions),
}

impl DaemonService {
    pub fn name(&self) -> String {
        match self {
            DaemonService::Setup => "setup".to_owned(),
            DaemonService::SetupDev => "setup_dev".to_owned(),
            DaemonService::Worker(opts) => match (opts.service, opts.price_provider) {
                (Some(service), Some(provider)) => format!("worker {} {}", service.as_ref(), provider.id()),
                (Some(service), None) => format!("worker {}", service.as_ref()),
                (None, _) => "worker all".to_owned(),
            },
            DaemonService::Parser(chain) => match chain {
                Some(chain) => format!("parser {}", chain.as_ref()),
                None => "parser".to_owned(),
            },
            DaemonService::Consumer(opts) => match (&opts.service, opts.indexer) {
                (Some(service), Some(indexer)) => format!("consumer {} {}", service.as_ref(), indexer.as_ref()),
                (Some(service), None) => format!("consumer {}", service.as_ref()),
                (None, _) => "consumer all".to_owned(),
            },
        }
    }
}

impl FromStr for DaemonService {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split_whitespace().collect();
        let name = parts.first().copied().ok_or_else(|| "Empty service name".to_string())?;

        match name {
            "setup" => Ok(DaemonService::Setup),
            "setup_dev" => Ok(DaemonService::SetupDev),
            "worker" => {
                let service = parts.get(1).map(|s| WorkerService::from_str(s).map_err(|_| format!("Invalid worker: {s}"))).transpose()?;
                let price_provider = if matches!(service, Some(WorkerService::Prices)) {
                    parts
                        .get(2)
                        .map(|s| PriceProvider::from_str(s).map_err(|_| format!("Invalid price provider: {s}")))
                        .transpose()?
                } else {
                    None
                };
                Ok(DaemonService::Worker(WorkerOptions { service, price_provider }))
            }
            "parser" => {
                let chain = parts.get(1).map(|s| Chain::from_str(s).map_err(|_| format!("Invalid chain: {s}"))).transpose()?;
                Ok(DaemonService::Parser(chain))
            }
            "consumer" => {
                let service = parts
                    .get(1)
                    .map(|s| ConsumerService::from_str(s).map_err(|_| format!("Invalid consumer: {s}")))
                    .transpose()?;
                let indexer = if matches!(service, Some(ConsumerService::Indexer)) {
                    parts
                        .get(2)
                        .map(|s| IndexerConsumer::from_str(s).map_err(|_| format!("Invalid indexer consumer: {s}")))
                        .transpose()?
                } else {
                    None
                };
                Ok(DaemonService::Consumer(ConsumerOptions { service, indexer }))
            }
            _ => Err(format!("Unknown service: {name}")),
        }
    }
}
