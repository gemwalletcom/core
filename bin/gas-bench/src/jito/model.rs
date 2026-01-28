use serde::Deserialize;

const LAMPORTS_PER_SOL: f64 = 1_000_000_000.0;
const MICRO_LAMPORTS_PER_LAMPORT: f64 = 1_000_000.0;

#[derive(Debug, Deserialize)]
pub struct JitoTipFloorEntry {
    pub landed_tips_25th_percentile: f64,
    pub landed_tips_50th_percentile: f64,
    pub landed_tips_75th_percentile: f64,
}

#[derive(Debug)]
pub struct JitoTipFloor {
    pub p25_lamports: u64,
    pub p50_lamports: u64,
    pub p75_lamports: u64,
}

impl JitoTipFloor {
    pub fn from_entry(entry: &JitoTipFloorEntry) -> Self {
        Self {
            p25_lamports: sol_to_lamports(entry.landed_tips_25th_percentile),
            p50_lamports: sol_to_lamports(entry.landed_tips_50th_percentile),
            p75_lamports: sol_to_lamports(entry.landed_tips_75th_percentile),
        }
    }
}

pub fn sol_to_lamports(sol: f64) -> u64 {
    (sol * LAMPORTS_PER_SOL) as u64
}

pub fn lamports_to_sol(lamports: u64) -> String {
    let sol = lamports as f64 / LAMPORTS_PER_SOL;
    if sol < 0.000001 {
        format!("{:.9}", sol)
    } else if sol < 0.001 {
        format!("{:.6}", sol)
    } else {
        format!("{:.4}", sol)
    }
}

pub fn priority_fee_to_lamports(micro_lamports_per_cu: u64, compute_units: u64) -> u64 {
    (micro_lamports_per_cu as u128 * compute_units as u128 / MICRO_LAMPORTS_PER_LAMPORT as u128) as u64
}

pub fn format_micro_lamports(micro_lamports: u64) -> String {
    if micro_lamports >= MICRO_LAMPORTS_PER_LAMPORT as u64 {
        format!("{:.2}M", micro_lamports as f64 / MICRO_LAMPORTS_PER_LAMPORT)
    } else if micro_lamports >= 1_000 {
        format!("{:.1}K", micro_lamports as f64 / 1_000.0)
    } else {
        format!("{}", micro_lamports)
    }
}
