use std::collections::{HashMap, VecDeque};
use std::time::Instant;

use primitives::Chain;
use tokio::sync::RwLock;

use crate::config::{AdaptiveMonitoringConfig, Url};

#[derive(Debug, Clone)]
pub struct HostHealthSnapshot {
    pub total: usize,
    pub errors: usize,
    pub ratio: f64,
    pub blocked_now: bool,
}

#[derive(Debug)]
pub struct RequestAdaptiveMonitor {
    config: AdaptiveMonitoringConfig,
    state: RwLock<HashMap<Chain, ChainAdaptiveState>>,
}

#[derive(Debug, Default)]
struct ChainAdaptiveState {
    hosts: HashMap<String, HostAdaptiveState>,
    last_switch_at: Option<Instant>,
}

#[derive(Debug)]
struct HostAdaptiveState {
    window: RequestWindow,
    blocked_until: Option<Instant>,
}

#[derive(Debug)]
struct RequestWindow {
    started_at: Instant,
    buckets: VecDeque<RequestBucket>,
    total: usize,
    errors: usize,
}

#[derive(Debug)]
struct RequestBucket {
    second: u64,
    total: usize,
    errors: usize,
}

impl RequestAdaptiveMonitor {
    pub fn new(config: AdaptiveMonitoringConfig) -> Self {
        Self {
            config,
            state: RwLock::new(HashMap::new()),
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    pub async fn reorder_urls(&self, chain: Chain, urls: &[Url]) -> Vec<Url> {
        if !self.config.enabled || urls.len() <= 1 {
            return urls.to_vec();
        }

        let now = Instant::now();
        let mut state = self.state.write().await;
        let chain_state = state.entry(chain).or_default();

        let mut available = Vec::with_capacity(urls.len());
        let mut blocked = Vec::new();

        for url in urls {
            let host = url.host();
            let host_state = chain_state.hosts.entry(host).or_insert_with(|| HostAdaptiveState::new(now));
            let (is_blocked, _) = host_state.refresh_block_state(&self.config, now);
            if is_blocked {
                blocked.push(url.clone());
            } else {
                available.push(url.clone());
            }
        }

        available.extend(blocked);
        available
    }

    pub async fn record_attempt(&self, chain: Chain, host: &str, has_error: bool) -> Option<HostHealthSnapshot> {
        if !self.config.enabled {
            return None;
        }

        let now = Instant::now();
        let mut state = self.state.write().await;
        let chain_state = state.entry(chain).or_default();
        let host_state = chain_state.hosts.entry(host.to_string()).or_insert_with(|| HostAdaptiveState::new(now));

        Some(host_state.record(&self.config, now, has_error))
    }

    pub async fn allow_switch_after_success(&self, chain: Chain, current_host: &str, new_host: &str) -> Option<HostHealthSnapshot> {
        if !self.config.enabled || current_host == new_host {
            return None;
        }

        let now = Instant::now();
        let mut state = self.state.write().await;
        let chain_state = state.entry(chain).or_default();

        let snapshot = {
            let current_state = chain_state.hosts.entry(current_host.to_string()).or_insert_with(|| HostAdaptiveState::new(now));
            let (is_blocked, _) = current_state.refresh_block_state(&self.config, now);
            if !is_blocked {
                return None;
            }

            current_state.snapshot(false)
        };

        if let Some(last_switch_at) = chain_state.last_switch_at
            && now.duration_since(last_switch_at) < self.config.min_switch_interval
        {
            return None;
        }

        Some(snapshot)
    }

    pub async fn mark_switch(&self, chain: Chain) {
        if !self.config.enabled {
            return;
        }

        let mut state = self.state.write().await;
        let chain_state = state.entry(chain).or_default();
        chain_state.last_switch_at = Some(Instant::now());
    }
}

impl HostAdaptiveState {
    fn new(now: Instant) -> Self {
        Self {
            window: RequestWindow::new(now),
            blocked_until: None,
        }
    }

    fn record(&mut self, config: &AdaptiveMonitoringConfig, now: Instant, has_error: bool) -> HostHealthSnapshot {
        self.window.record(config, now, has_error);
        let (_, blocked_now) = self.refresh_block_state(config, now);
        self.snapshot(blocked_now)
    }

    fn snapshot(&self, blocked_now: bool) -> HostHealthSnapshot {
        HostHealthSnapshot {
            total: self.window.total,
            errors: self.window.errors,
            ratio: self.window.error_ratio(),
            blocked_now,
        }
    }

    fn refresh_block_state(&mut self, config: &AdaptiveMonitoringConfig, now: Instant) -> (bool, bool) {
        self.window.prune(config, now);

        let was_blocked = self.blocked_until.is_some_and(|until| until > now);
        let was_previously_blocked = self.blocked_until.is_some();
        let enough_samples = self.window.total >= config.min_samples;
        let ratio = self.window.error_ratio();

        if enough_samples && ratio >= config.error_threshold {
            self.blocked_until = Some(now + config.cooldown);
            return (true, !was_blocked);
        }

        if was_blocked {
            return (true, false);
        }

        if was_previously_blocked && enough_samples && ratio > config.recovery_threshold {
            self.blocked_until = Some(now + config.cooldown);
            return (true, true);
        }

        if was_previously_blocked {
            self.blocked_until = None;
        }

        (false, false)
    }
}

impl RequestWindow {
    fn new(now: Instant) -> Self {
        Self {
            started_at: now,
            buckets: VecDeque::new(),
            total: 0,
            errors: 0,
        }
    }

    fn record(&mut self, config: &AdaptiveMonitoringConfig, now: Instant, has_error: bool) {
        self.prune(config, now);
        let second = self.second(now);

        if let Some(last) = self.buckets.back_mut()
            && last.second == second
        {
            last.total += 1;
            if has_error {
                last.errors += 1;
            }
            self.total += 1;
            if has_error {
                self.errors += 1;
            }
            return;
        }

        self.buckets.push_back(RequestBucket {
            second,
            total: 1,
            errors: if has_error { 1 } else { 0 },
        });
        self.total += 1;
        if has_error {
            self.errors += 1;
        }
    }

    fn prune(&mut self, config: &AdaptiveMonitoringConfig, now: Instant) {
        let second = self.second(now);
        let window_seconds = std::cmp::max(config.window.as_secs(), 1);

        while let Some(front) = self.buckets.front() {
            if second.saturating_sub(front.second) < window_seconds {
                break;
            }

            if let Some(expired) = self.buckets.pop_front() {
                self.total = self.total.saturating_sub(expired.total);
                self.errors = self.errors.saturating_sub(expired.errors);
            }
        }
    }

    fn second(&self, now: Instant) -> u64 {
        now.duration_since(self.started_at).as_secs()
    }

    fn error_ratio(&self) -> f64 {
        if self.total == 0 {
            return 0.0;
        }

        self.errors as f64 / self.total as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testkit::config as testkit;
    use crate::testkit::sync::url;
    use std::time::Duration;

    fn config() -> AdaptiveMonitoringConfig {
        AdaptiveMonitoringConfig {
            min_samples: 4,
            cooldown: Duration::from_secs(60),
            min_switch_interval: Duration::from_secs(30),
            ..testkit::adaptive_monitoring_config()
        }
    }

    #[tokio::test]
    async fn opens_host_when_error_threshold_is_reached() {
        let monitor = RequestAdaptiveMonitor::new(config());
        let host = "a.example.com";

        let _ = monitor.record_attempt(Chain::Ethereum, host, true).await;
        let _ = monitor.record_attempt(Chain::Ethereum, host, true).await;
        let _ = monitor.record_attempt(Chain::Ethereum, host, false).await;
        let snapshot = monitor.record_attempt(Chain::Ethereum, host, false).await.unwrap();

        assert_eq!(snapshot.total, 4);
        assert_eq!(snapshot.errors, 2);
        assert!(snapshot.blocked_now);
    }

    #[tokio::test]
    async fn reorders_urls_to_push_blocked_host_to_the_end() {
        let monitor = RequestAdaptiveMonitor::new(config());
        let host = "a.example.com";
        let _ = monitor.record_attempt(Chain::Ethereum, host, true).await;
        let _ = monitor.record_attempt(Chain::Ethereum, host, true).await;
        let _ = monitor.record_attempt(Chain::Ethereum, host, false).await;
        let _ = monitor.record_attempt(Chain::Ethereum, host, false).await;

        let urls = vec![url("https://a.example.com/rpc"), url("https://b.example.com/rpc"), url("https://c.example.com/rpc")];

        let reordered = monitor.reorder_urls(Chain::Ethereum, &urls).await;
        assert_eq!(reordered[0].host(), "b.example.com");
        assert_eq!(reordered[2].host(), "a.example.com");
    }

    #[tokio::test]
    async fn allows_switch_when_blocked_until_switch_is_marked() {
        let monitor = RequestAdaptiveMonitor::new(config());
        let chain = Chain::Ethereum;
        let current = "a.example.com";
        let next = "b.example.com";

        assert!(monitor.allow_switch_after_success(chain, current, next).await.is_none());

        let _ = monitor.record_attempt(chain, current, true).await;
        let _ = monitor.record_attempt(chain, current, true).await;
        let _ = monitor.record_attempt(chain, current, false).await;
        let _ = monitor.record_attempt(chain, current, false).await;

        assert!(monitor.allow_switch_after_success(chain, current, next).await.is_some());
        assert!(monitor.allow_switch_after_success(chain, current, next).await.is_some());
        monitor.mark_switch(chain).await;
        assert!(monitor.allow_switch_after_success(chain, current, next).await.is_none());
    }

    #[tokio::test]
    async fn disabled_monitor_keeps_original_order() {
        let adaptive = AdaptiveMonitoringConfig { enabled: false, ..config() };
        let monitor = RequestAdaptiveMonitor::new(adaptive);

        let urls = vec![url("https://a.example.com/rpc"), url("https://b.example.com/rpc")];
        let reordered = monitor.reorder_urls(Chain::Ethereum, &urls).await;

        assert_eq!(reordered[0].url, urls[0].url);
        assert_eq!(reordered[1].url, urls[1].url);
        assert!(monitor.record_attempt(Chain::Ethereum, "a.example.com", true).await.is_none());
    }
}
