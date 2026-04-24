use crate::model::WorkerService;
use primitives::{Chain, ConfigKey, ConfigParamKey, FiatProviderName, PlatformStore, PriceProvider};
use std::error::Error;
use std::time::Duration;
use storage::ConfigCacher;
use strum::AsRefStr;

#[derive(Clone, Debug)]
enum JobInterval {
    Config(ConfigKey),
}

impl JobInterval {
    fn resolve(self, config: Option<&ConfigCacher>) -> Result<Duration, Box<dyn Error + Send + Sync>> {
        match self {
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

pub trait JobLabel {
    fn job_label(&self) -> String;
}

impl JobLabel for str {
    fn job_label(&self) -> String {
        self.to_string()
    }
}

impl JobLabel for String {
    fn job_label(&self) -> String {
        self.clone()
    }
}

impl<T> JobLabel for &T
where
    T: JobLabel + ?Sized,
{
    fn job_label(&self) -> String {
        (*self).job_label()
    }
}

impl JobLabel for Chain {
    fn job_label(&self) -> String {
        self.as_ref().to_string()
    }
}

impl JobLabel for FiatProviderName {
    fn job_label(&self) -> String {
        self.as_ref().to_string()
    }
}

impl JobLabel for PlatformStore {
    fn job_label(&self) -> String {
        self.as_ref().to_string()
    }
}

impl JobLabel for primitives::SwapProvider {
    fn job_label(&self) -> String {
        self.as_ref().to_string()
    }
}

impl JobLabel for PriceProvider {
    fn job_label(&self) -> String {
        self.id().to_string()
    }
}

fn compose_job_name(base: &str, label: Option<&str>) -> String {
    match label.map(str::trim).filter(|value| !value.is_empty()) {
        Some(suffix) => format!("{base}.{suffix}"),
        None => base.to_string(),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, AsRefStr)]
#[strum(serialize_all = "snake_case")]
pub enum WorkerJob {
    AlertPriceAlerts,
    UpdateSuspiciousAssetRanks,
    UpdateStakeApy,
    UpdatePerpetuals,
    UpdateUsageRanks,
    UpdateAssetsImages,
    UpdateAssetsHasPrice,
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
    UpdateStoreVersion,
    UpdateChainValidators,
    UpdateValidatorsFromStaticAssets,
    CheckRewardsAbuse,
    CheckRewardsEligibility,
    CleanupOutdatedAssets,
    UpdateFiatRates,
    UpdatePricesTop,
    UpdatePricesHigh,
    UpdatePricesLow,
    UpdatePrices,
    AggregateHourlyCharts,
    AggregateDailyCharts,
    CleanupChartsRaw,
    CleanupChartsHourly,
    UpdateMarkets,
    UpdatePricesMetrics,
    UpdateChartsHistory,
    UpdateObservedPrices,
    UpdatePricesAssets,
    UpdatePricesAssetsNew,
    UpdatePricesAssetsMetadata,
    UpdateInTransitTransactions,
    UpdatePendingTransactions,
    UpdateSwapVaultAddresses,
    AlertStakeRewards,
    ClassifyPerpetualAddresses,
    ObservePerpetualActiveAddresses,
    ObservePerpetualPriorityAddresses,
    RefreshPerpetualTrackedAddresses,
}

impl WorkerJob {
    fn spec(&self) -> JobSpec {
        use WorkerJob::*;
        match self {
            AlertPriceAlerts => JobSpec::new(WorkerService::Alerter, JobInterval::Config(ConfigKey::AlerterPriceAlertsTimer)),
            UpdateSuspiciousAssetRanks => JobSpec::new(WorkerService::Assets, JobInterval::Config(ConfigKey::AssetsTimerUpdateSuspicious)),
            UpdateStakeApy => JobSpec::new(WorkerService::Assets, JobInterval::Config(ConfigKey::AssetsTimerUpdateStakeApy)),
            UpdatePerpetuals => JobSpec::new(WorkerService::Assets, JobInterval::Config(ConfigKey::AssetsTimerUpdatePerpetuals)),
            UpdateUsageRanks => JobSpec::new(WorkerService::Assets, JobInterval::Config(ConfigKey::AssetsTimerUpdateUsageRank)),
            UpdateAssetsImages => JobSpec::new(WorkerService::Assets, JobInterval::Config(ConfigKey::AssetsTimerUpdateImages)),
            UpdateAssetsHasPrice => JobSpec::new(WorkerService::Assets, JobInterval::Config(ConfigKey::AssetsTimerUpdateHasPrice)),
            CleanupStaleDeviceSubscriptions => JobSpec::new(WorkerService::System, JobInterval::Config(ConfigKey::DeviceTimerUpdater)),
            ObserveInactiveDevices => JobSpec::new(WorkerService::System, JobInterval::Config(ConfigKey::DeviceTimerInactiveObserver)),
            UpdateFiatAssets => JobSpec::new(WorkerService::Fiat, JobInterval::Config(ConfigKey::FiatTimerUpdateAssets)),
            UpdateFiatProviderCountries => JobSpec::new(WorkerService::Fiat, JobInterval::Config(ConfigKey::FiatTimerUpdateProviderCountries)),
            UpdateFiatBuyableAssets => JobSpec::new(WorkerService::Fiat, JobInterval::Config(ConfigKey::FiatTimerUpdateBuyableAssets)),
            UpdateFiatSellableAssets => JobSpec::new(WorkerService::Fiat, JobInterval::Config(ConfigKey::FiatTimerUpdateSellableAssets)),
            UpdateTrendingFiatAssets => JobSpec::new(WorkerService::Fiat, JobInterval::Config(ConfigKey::FiatTimerUpdateTrending)),
            UpdateAssetsIndex => JobSpec::new(WorkerService::Search, JobInterval::Config(ConfigKey::SearchAssetsUpdateInterval)),
            UpdatePerpetualsIndex => JobSpec::new(WorkerService::Search, JobInterval::Config(ConfigKey::SearchPerpetualsUpdateInterval)),
            UpdateNftsIndex => JobSpec::new(WorkerService::Search, JobInterval::Config(ConfigKey::SearchNftsUpdateInterval)),
            CleanupProcessedTransactions => JobSpec::new(WorkerService::System, JobInterval::Config(ConfigKey::TransactionTimerCleanup)),
            UpdateStoreVersion => JobSpec::new(WorkerService::System, JobInterval::Config(ConfigKey::VersionTimerUpdateStoreVersions)),
            UpdateChainValidators => JobSpec::new(WorkerService::Assets, JobInterval::Config(ConfigKey::ScanTimerUpdateValidators)),
            UpdateValidatorsFromStaticAssets => JobSpec::new(WorkerService::Assets, JobInterval::Config(ConfigKey::ScanTimerUpdateValidatorsStatic)),
            CheckRewardsAbuse => JobSpec::new(WorkerService::Rewards, JobInterval::Config(ConfigKey::RewardsTimerAbuseChecker)),
            CheckRewardsEligibility => JobSpec::new(WorkerService::Rewards, JobInterval::Config(ConfigKey::RewardsTimerEligibilityChecker)),
            CleanupOutdatedAssets => JobSpec::new(WorkerService::Prices, JobInterval::Config(ConfigKey::PriceTimerCleanOutdated)),
            UpdateFiatRates => JobSpec::new(WorkerService::Fiat, JobInterval::Config(ConfigKey::PriceTimerFiatRates)),
            UpdatePricesTop => JobSpec::new(WorkerService::Prices, JobInterval::Config(ConfigKey::PriceTimerTopMarketCap)),
            UpdatePricesHigh => JobSpec::new(WorkerService::Prices, JobInterval::Config(ConfigKey::PriceTimerHighMarketCap)),
            UpdatePricesLow => JobSpec::new(WorkerService::Prices, JobInterval::Config(ConfigKey::PriceTimerLowMarketCap)),
            UpdatePrices => JobSpec::new(WorkerService::Prices, JobInterval::Config(ConfigKey::PriceTimerPrices)),
            AggregateHourlyCharts => JobSpec::new(WorkerService::Prices, JobInterval::Config(ConfigKey::PriceTimerChartsHourly)),
            AggregateDailyCharts => JobSpec::new(WorkerService::Prices, JobInterval::Config(ConfigKey::PriceTimerChartsDaily)),
            CleanupChartsRaw => JobSpec::new(WorkerService::Prices, JobInterval::Config(ConfigKey::PriceTimerCleanupChartsRaw)),
            CleanupChartsHourly => JobSpec::new(WorkerService::Prices, JobInterval::Config(ConfigKey::PriceTimerCleanupChartsHourly)),
            UpdateMarkets => JobSpec::new(WorkerService::Prices, JobInterval::Config(ConfigKey::PriceTimerMarkets)),
            UpdatePricesMetrics => JobSpec::new(WorkerService::Prices, JobInterval::Config(ConfigKey::PriceTimerMetrics)),
            UpdateChartsHistory => JobSpec::new(WorkerService::Prices, JobInterval::Config(ConfigKey::PriceTimerChartsHistory)),
            UpdateObservedPrices => JobSpec::new(WorkerService::Prices, JobInterval::Config(ConfigKey::PriceObservedFetchInterval)),
            UpdatePricesAssets => JobSpec::new(WorkerService::Prices, JobInterval::Config(ConfigKey::PriceTimerAssets)),
            UpdatePricesAssetsNew => JobSpec::new(WorkerService::Prices, JobInterval::Config(ConfigKey::PriceTimerAssetsNew)),
            UpdatePricesAssetsMetadata => JobSpec::new(WorkerService::Prices, JobInterval::Config(ConfigKey::PriceTimerAssetsMetadata)),
            UpdateInTransitTransactions => JobSpec::new(WorkerService::Transactions, JobInterval::Config(ConfigKey::TransactionTimerInTransitUpdate)),
            UpdatePendingTransactions => JobSpec::new(WorkerService::Transactions, JobInterval::Config(ConfigKey::TransactionTimerPendingUpdate)),
            UpdateSwapVaultAddresses => JobSpec::new(WorkerService::Transactions, JobInterval::Config(ConfigKey::TransactionTimerSwapVaultAddresses)),
            AlertStakeRewards => JobSpec::new(WorkerService::Alerter, JobInterval::Config(ConfigKey::AlerterStakeRewardsTimer)),
            ClassifyPerpetualAddresses => JobSpec::new(WorkerService::Perpetuals, JobInterval::Config(ConfigKey::PerpetualClassifierInterval)),
            ObservePerpetualActiveAddresses => JobSpec::new(WorkerService::Perpetuals, JobInterval::Config(ConfigKey::PerpetualObserverInterval)),
            ObservePerpetualPriorityAddresses => JobSpec::new(WorkerService::Perpetuals, JobInterval::Config(ConfigKey::PerpetualPriorityObserverInterval)),
            RefreshPerpetualTrackedAddresses => JobSpec::new(WorkerService::Perpetuals, JobInterval::Config(ConfigKey::PerpetualAddressRefreshInterval)),
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
pub struct JobVariant {
    job: WorkerJob,
    label: Option<String>,
    override_interval: Option<Duration>,
}

impl JobVariant {
    pub fn new(job: WorkerJob) -> Self {
        Self {
            job,
            label: None,
            override_interval: None,
        }
    }

    pub fn labeled(job: WorkerJob, label: impl JobLabel) -> Self {
        Self {
            job,
            label: Some(label.job_label()),
            override_interval: None,
        }
    }

    pub fn every(mut self, interval: Duration) -> Self {
        self.override_interval = Some(interval);
        self
    }

    pub fn with_param_duration(self, config: &ConfigCacher, key: &ConfigParamKey) -> Result<Self, storage::DatabaseError> {
        Ok(self.every(config.get_param_duration(key)?))
    }

    pub fn name(&self) -> String {
        job_name(self.job, self.label.as_deref())
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

impl From<WorkerJob> for JobVariant {
    fn from(job: WorkerJob) -> Self {
        JobVariant::new(job)
    }
}

pub fn job_name(job: WorkerJob, label: Option<&str>) -> String {
    compose_job_name(job.as_ref(), label)
}
