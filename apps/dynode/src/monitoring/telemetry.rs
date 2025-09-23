use std::error::Error;

use gem_tracing::{error_with_fields, error_with_fields_impl, info_with_fields_impl, DurationMs};
use primitives::NodeStatusState;

use crate::config::{Domain, Url};

use super::sync::{NodeStatusObservation, NodeSyncAnalyzer};

pub struct NodeTelemetry;

impl NodeTelemetry {
    pub fn log_status_debug(domain: &Domain, observations: &[NodeStatusObservation]) {
        let chain = domain.chain.as_ref();
        for observation in observations {
            match &observation.state {
                NodeStatusState::Healthy(sync_status) => {
                    let mut fields = vec![("host", observation.url.host())];
                    if !sync_status.in_sync {
                        fields.push(("in_sync", sync_status.in_sync.to_string()));
                    }

                    let latency = DurationMs(observation.latency);
                    let latest = sync_status.latest_block_number;
                    let current = if sync_status.in_sync { None } else { sync_status.current_block_number };

                    log_info_event("Node check", chain, fields, &latency, latest, current);
                }
                NodeStatusState::Error { message } => {
                    let latency = DurationMs(observation.latency);
                    log_info_event(
                        "Node check",
                        chain,
                        [("host", observation.url.host()), ("message", message.clone())],
                        &latency,
                        None,
                        None,
                    );
                }
            }
        }
    }

    pub fn log_node_healthy(domain: &Domain, observation: &NodeStatusObservation) {
        let chain = domain.chain.as_ref();
        if let NodeStatusState::Healthy(status) = &observation.state {
            let mut fields = vec![("host", observation.url.host())];
            if !status.in_sync {
                fields.push(("in_sync", status.in_sync.to_string()));
            }

            let latency = DurationMs(observation.latency);
            let latest = status.latest_block_number;
            let current = if status.in_sync { None } else { status.current_block_number };

            log_info_event("Node ok", chain, fields, &latency, latest, current);
        }
    }

    pub fn log_node_unhealthy(domain: &Domain, observation: &NodeStatusObservation) {
        let chain = domain.chain.as_ref();
        match &observation.state {
            NodeStatusState::Healthy(status) => {
                let mut fields = vec![("host", observation.url.host())];
                if !status.in_sync {
                    fields.push(("in_sync", status.in_sync.to_string()));
                }

                let latency = DurationMs(observation.latency);
                let latest = status.latest_block_number;
                let current = if status.in_sync { None } else { status.current_block_number };
                let error = std::io::Error::other("Current node not in sync");

                log_error_event("Node out of sync", &error, chain, fields, &latency, latest, current);
            }
            NodeStatusState::Error { message } => {
                let latency = DurationMs(observation.latency);
                let error = std::io::Error::other(message.clone());
                log_error_event(
                    "Node check error",
                    &error,
                    chain,
                    [("host", observation.url.host()), ("message", message.clone())],
                    &latency,
                    None,
                    None,
                );
            }
        }
    }

    pub fn log_node_switch(domain: &Domain, previous: &Url, observation: &NodeStatusObservation) {
        let chain = domain.chain.as_ref();
        match &observation.state {
            NodeStatusState::Healthy(status) => {
                let latency = DurationMs(observation.latency);
                let latest = status.latest_block_number;
                let current = if status.in_sync { None } else { status.current_block_number };

                log_info_event(
                    "Node switch",
                    chain,
                    [("new_host", observation.url.host()), ("old_host", previous.host())],
                    &latency,
                    latest,
                    current,
                );
            }
            NodeStatusState::Error { .. } => {
                let latency = DurationMs(observation.latency);
                log_info_event(
                    "Node switch",
                    chain,
                    [("new_host", observation.url.host()), ("old_host", previous.host())],
                    &latency,
                    None,
                    None,
                );
            }
        }
    }

    pub fn log_no_candidate(domain: &Domain, observations: &[NodeStatusObservation]) {
        error_with_fields!(
            "Node switch unavailable",
            &std::io::Error::other("No healthy nodes available"),
            chain = domain.chain.as_ref(),
            statuses = &NodeSyncAnalyzer::format_status_summary(observations),
        );
    }

    pub fn log_monitor_error(domain: &Domain, err: &dyn std::error::Error) {
        error_with_fields!("Node monitor error", err, chain = domain.chain.as_ref());
    }

    pub fn log_missing_current(domain: &Domain) {
        error_with_fields!(
            "Node monitor current missing",
            &std::io::Error::other("Node not configured"),
            chain = domain.chain.as_ref(),
        );
    }
}

fn log_info_event<I>(message: &'static str, chain: &str, fields: I, latency: &DurationMs, latest: Option<u64>, current: Option<u64>)
where
    I: IntoIterator<Item = (&'static str, String)>,
{
    emit_event(message, chain, fields, latency, latest, current, |msg, slice| info_with_fields_impl(msg, slice));
}

fn log_error_event<I>(message: &'static str, err: &dyn Error, chain: &str, fields: I, latency: &DurationMs, latest: Option<u64>, current: Option<u64>)
where
    I: IntoIterator<Item = (&'static str, String)>,
{
    emit_event(message, chain, fields, latency, latest, current, |msg, slice| {
        error_with_fields_impl(msg, err, slice)
    });
}

fn emit_event<I>(
    message: &'static str,
    chain: &str,
    fields: I,
    latency: &DurationMs,
    latest: Option<u64>,
    current: Option<u64>,
    sink: impl Fn(&'static str, &[(&str, &dyn std::fmt::Display)]),
) where
    I: IntoIterator<Item = (&'static str, String)>,
{
    let mut values: Vec<(&'static str, String)> = fields.into_iter().collect();
    if let Some(latest) = latest {
        values.push(("latest_block", latest.to_string()));
    }
    if let Some(current) = current {
        values.push(("current_block", current.to_string()));
    }

    let mut display: Vec<(&str, &dyn std::fmt::Display)> = Vec::with_capacity(values.len() + 2);
    display.push(("chain", &chain));
    for (key, value) in &values {
        display.push((*key, value));
    }
    display.push(("latency", latency));

    sink(message, &display);
}
