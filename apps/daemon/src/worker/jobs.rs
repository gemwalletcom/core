use crate::model::WorkerService;
use primitives::ConfigKey;
use std::error::Error;
use std::time::Duration;
use storage::ConfigCacher;
use strum::AsRefStr;

#[derive(Clone, Debug)]
enum JobInterval {
    Config(ConfigKey),
    Duration(Duration),
}

impl JobInterval {
    fn resolve(self, config: Option<&ConfigCacher>) -> Result<Duration, Box<dyn Error + Send + Sync>> {
        match self {
            JobInterval::Duration(duration) => Ok(duration),
            JobInterval::Config(key) => {
                let cfg = config.ok_or_else(|| format!("ConfigCacher required for {:?}", key))?;
                Ok(cfg.get_duration(key)?)
            }
        }
    }
}

#[derive(Clone, Debug)]
struct JobSpec {
    worker: WorkerService,
    interval: JobInterval,
}

impl JobSpec {
    const fn new(worker: WorkerService, interval: JobInterval) -> Self {
        Self { worker, interval }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, AsRefStr)]
#[strum(serialize_all = "snake_case")]
pub enum WorkerJob {
    SendPriceAlerts,
    UpdateExistingPricesAssets,
    UpdateAllPricesAssets,
    UpdateNativePricesAssets,
    UpdateCoingeckoTrendingAssets,
    UpdateCoingeckoRecentlyAddedAssets,
    UpdateSuspiciousAssetRanks,
    UpdateStakingApy,
    UpdatePerpetuals,
    UpdateUsageRanks,
    UpdateAssetsImages,
    CleanupStaleDeviceSubscriptions,
    ObserveInactiveDevices,
    UpdateFiatAssets,
    UpdateFiatProviderCountries,
    UpdateFiatBuyableAssets,
    UpdateFiatSellableAssets,
    UpdateTrendingFiatAssets,
    UpdateAssetsIndex,
    UpdatePerpetualsIndex,
    UpdateNftsIndex,
    CleanupProcessedTransactions,
    UpdateStoreVersions,
    UpdateChainValidators,
    UpdateValidatorsFromStaticAssets,
    CheckRewardsAbuse,
    CleanupOutdatedAssets,
    UpdateFiatRates,
    UpdatePricesTopMarketCap,
    UpdatePricesHighMarketCap,
    UpdatePricesLowMarketCap,
    AggregateHourlyCharts,
    AggregateDailyCharts,
    CleanupChartsData,
    UpdateMarkets,
    UpdateObservedPrices,
    UpdateDexFeeds,
    UpdateDexPrices,
}

impl WorkerJob {
    fn spec(&self) -> JobSpec {
        use WorkerJob::*;
        match self {
            SendPriceAlerts => JobSpec::new(WorkerService::Alerter, JobInterval::Config(ConfigKey::AlerterInterval)),
            UpdateExistingPricesAssets => JobSpec::new(WorkerService::Assets, JobInterval::Config(ConfigKey::AssetsTimerUpdateExisting)),
            UpdateAllPricesAssets => JobSpec::new(WorkerService::Assets, JobInterval::Config(ConfigKey::AssetsTimerUpdateAll)),
            UpdateNativePricesAssets => JobSpec::new(WorkerService::Assets, JobInterval::Config(ConfigKey::AssetsTimerUpdateNative)),
            UpdateCoingeckoTrendingAssets => JobSpec::new(WorkerService::Assets, JobInterval::Config(ConfigKey::AssetsTimerUpdateTrending)),
            UpdateCoingeckoRecentlyAddedAssets => JobSpec::new(WorkerService::Assets, JobInterval::Config(ConfigKey::AssetsTimerUpdateRecentlyAdded)),
            UpdateSuspiciousAssetRanks => JobSpec::new(WorkerService::Assets, JobInterval::Config(ConfigKey::AssetsTimerUpdateSuspicious)),
            UpdateStakingApy => JobSpec::new(WorkerService::Assets, JobInterval::Config(ConfigKey::AssetsTimerUpdateStakingApy)),
            UpdatePerpetuals => JobSpec::new(WorkerService::Assets, JobInterval::Config(ConfigKey::AssetsTimerUpdatePerpetuals)),
            UpdateUsageRanks => JobSpec::new(WorkerService::Assets, JobInterval::Config(ConfigKey::AssetsTimerUpdateUsageRank)),
            UpdateAssetsImages => JobSpec::new(WorkerService::Assets, JobInterval::Config(ConfigKey::AssetsTimerUpdateImages)),
            CleanupStaleDeviceSubscriptions => JobSpec::new(WorkerService::Device, JobInterval::Config(ConfigKey::DeviceTimerUpdater)),
            ObserveInactiveDevices => JobSpec::new(WorkerService::Device, JobInterval::Config(ConfigKey::DeviceTimerInactiveObserver)),
            UpdateFiatAssets => JobSpec::new(WorkerService::Fiat, JobInterval::Config(ConfigKey::FiatTimerUpdateAssets)),
            UpdateFiatProviderCountries => JobSpec::new(WorkerService::Fiat, JobInterval::Config(ConfigKey::FiatTimerUpdateProviderCountries)),
            UpdateFiatBuyableAssets => JobSpec::new(WorkerService::Fiat, JobInterval::Config(ConfigKey::FiatTimerUpdateBuyableAssets)),
            UpdateFiatSellableAssets => JobSpec::new(WorkerService::Fiat, JobInterval::Config(ConfigKey::FiatTimerUpdateBuyableAssets)),
            UpdateTrendingFiatAssets => JobSpec::new(WorkerService::Fiat, JobInterval::Config(ConfigKey::FiatTimerUpdateTrending)),
            UpdateAssetsIndex => JobSpec::new(WorkerService::Search, JobInterval::Config(ConfigKey::SearchAssetsUpdateInterval)),
            UpdatePerpetualsIndex => JobSpec::new(WorkerService::Search, JobInterval::Config(ConfigKey::SearchPerpetualsUpdateInterval)),
            UpdateNftsIndex => JobSpec::new(WorkerService::Search, JobInterval::Config(ConfigKey::SearchNftsUpdateInterval)),
            CleanupProcessedTransactions => JobSpec::new(WorkerService::Transaction, JobInterval::Config(ConfigKey::TransactionTimerUpdater)),
            UpdateStoreVersions => JobSpec::new(WorkerService::Version, JobInterval::Config(ConfigKey::VersionTimerUpdateStoreVersions)),
            UpdateChainValidators => JobSpec::new(WorkerService::Scan, JobInterval::Config(ConfigKey::ScanTimerUpdateValidators)),
            UpdateValidatorsFromStaticAssets => JobSpec::new(WorkerService::Scan, JobInterval::Config(ConfigKey::ScanTimerUpdateValidatorsStatic)),
            CheckRewardsAbuse => JobSpec::new(WorkerService::Rewards, JobInterval::Config(ConfigKey::RewardsTimerAbuseChecker)),
            CleanupOutdatedAssets => JobSpec::new(WorkerService::Pricer, JobInterval::Config(ConfigKey::PriceTimerCleanOutdated)),
            UpdateFiatRates => JobSpec::new(WorkerService::Pricer, JobInterval::Config(ConfigKey::PriceTimerFiatRates)),
            UpdatePricesTopMarketCap => JobSpec::new(WorkerService::Pricer, JobInterval::Config(ConfigKey::PriceTimerTopMarketCap)),
            UpdatePricesHighMarketCap => JobSpec::new(WorkerService::Pricer, JobInterval::Config(ConfigKey::PriceTimerHighMarketCap)),
            UpdatePricesLowMarketCap => JobSpec::new(WorkerService::Pricer, JobInterval::Config(ConfigKey::PriceTimerLowMarketCap)),
            AggregateHourlyCharts => JobSpec::new(WorkerService::Pricer, JobInterval::Config(ConfigKey::PriceTimerChartsHourly)),
            AggregateDailyCharts => JobSpec::new(WorkerService::Pricer, JobInterval::Config(ConfigKey::PriceTimerChartsDaily)),
            CleanupChartsData => JobSpec::new(WorkerService::Pricer, JobInterval::Config(ConfigKey::PriceTimerCleanupCharts)),
            UpdateMarkets => JobSpec::new(WorkerService::Pricer, JobInterval::Config(ConfigKey::PriceTimerMarkets)),
            UpdateObservedPrices => JobSpec::new(WorkerService::Pricer, JobInterval::Config(ConfigKey::PriceObservedFetchInterval)),
            UpdateDexFeeds => JobSpec::new(WorkerService::PricesDex, JobInterval::Duration(Duration::from_secs(3600))),
            UpdateDexPrices => JobSpec::new(WorkerService::PricesDex, JobInterval::Duration(Duration::from_secs(1800))),
        }
    }

    pub fn worker(&self) -> WorkerService {
        self.spec().worker
    }

    fn interval(&self) -> JobInterval {
        self.spec().interval
    }
}

#[derive(Clone, Debug)]
pub struct JobInstance {
    job: WorkerJob,
    name: String,
    override_interval: Option<Duration>,
}

impl JobInstance {
    pub fn new(job: WorkerJob) -> Self {
        Self {
            name: job.as_ref().to_string(),
            job,
            override_interval: None,
        }
    }

    pub fn labeled(job: WorkerJob, label: impl Into<String>) -> Self {
        let label = label.into();
        let trimmed = label.trim();
        let name = if trimmed.is_empty() {
            job.as_ref().to_string()
        } else {
            format!("{}.{}", job.as_ref(), trimmed)
        };
        Self {
            job,
            name,
            override_interval: None,
        }
    }

    pub fn every(mut self, interval: Duration) -> Self {
        self.override_interval = Some(interval);
        self
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn worker(&self) -> WorkerService {
        self.job.worker()
    }

    pub fn resolve_interval(&self, config: Option<&ConfigCacher>) -> Result<Duration, Box<dyn Error + Send + Sync>> {
        if let Some(duration) = self.override_interval {
            Ok(duration)
        } else {
            self.job.interval().resolve(config)
        }
    }
}

impl From<WorkerJob> for JobInstance {
    fn from(job: WorkerJob) -> Self {
        JobInstance::new(job)
    }
}
