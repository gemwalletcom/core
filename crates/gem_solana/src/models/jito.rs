use primitives::{SolanaAccountMeta, SolanaInstruction};
use rand::seq::IndexedRandom;
use solana_primitives::Pubkey;

use crate::SYSTEM_PROGRAM_ID;

pub const JITO_TIP_MIN_LAMPORTS: u64 = 10_000; // 0.00001 SOL

pub const JITO_TIP_ACCOUNTS: [&str; 8] = [
    "96gYZGLnJYVFmbjzopPSU6QiEV5fGqZNyN9nmNhvrZU5",
    "HFqU5x63VTqvQss8hp11i4wVV8bD44PvwucfZ2bU7gRe",
    "Cw8CFyM9FkoMi7K7Crf6HNQqf4uEMzpKw6QNghXLvLkY",
    "ADaUMid9yfUytqMBgopwjb2DTLSokTSzL1zt6iGPaS49",
    "DfXygSm4jCyNCybVYYK6DwvWqjKee8pbDmJGcLWNDXjh",
    "ADuUkR4vqLUMWXxW9gh6D6L8pMSawimctcNZ5pGwDcEt",
    "DttWaMuVvTiduZRnguLF7jNxTgiMBZ1hyAumKUiL2KRL",
    "3AVi9Tg9Uo68tJfuvoKvqKNWKkC5wPdSSdeBnizKZ6jT",
];

#[derive(Debug, Clone)]
pub struct JitoTipEstimates {
    pub slow: u64,
    pub normal: u64,
    pub fast: u64,
}

#[derive(Debug, Clone, Default)]
pub struct FeeStats {
    pub median: i64,
    pub p75: i64,
    pub p90: i64,
    pub avg: i64,
    pub count: usize,
}

pub fn estimate_jito_tips(stats: &FeeStats) -> JitoTipEstimates {
    const BASE_SLOW: u64 = 1_000;
    const BASE_NORMAL: u64 = 3_000;
    const BASE_FAST: u64 = 10_000;
    const REFERENCE_FEE: f64 = 10_000.0;

    let congestion_multiplier = if stats.avg > 0 {
        let raw_multiplier = stats.avg as f64 / REFERENCE_FEE;
        (1.0 + raw_multiplier.sqrt()).clamp(1.0, 10.0)
    } else {
        1.0
    };

    JitoTipEstimates {
        slow: ((BASE_SLOW as f64 * congestion_multiplier) as u64).max(JITO_TIP_MIN_LAMPORTS),
        normal: ((BASE_NORMAL as f64 * congestion_multiplier) as u64).max(JITO_TIP_MIN_LAMPORTS),
        fast: ((BASE_FAST as f64 * congestion_multiplier) as u64).max(JITO_TIP_MIN_LAMPORTS),
    }
}

fn percentile(sorted_values: &[i64], p: usize) -> i64 {
    if sorted_values.is_empty() {
        return 0;
    }
    let idx = (p * sorted_values.len() / 100).min(sorted_values.len() - 1);
    sorted_values[idx]
}

pub fn calculate_fee_stats(fees: &[i64]) -> FeeStats {
    if fees.is_empty() {
        return FeeStats::default();
    }

    let mut values = fees.to_vec();
    values.sort();

    let count = values.len();
    let sum: i64 = values.iter().sum();

    FeeStats {
        median: percentile(&values, 50),
        p75: percentile(&values, 75),
        p90: percentile(&values, 90),
        avg: sum / count as i64,
        count,
    }
}

fn random_tip_account() -> &'static str {
    JITO_TIP_ACCOUNTS.choose(&mut rand::rng()).unwrap()
}

#[cfg(feature = "signer")]
pub fn random_tip_pubkey() -> solana_primitives::Pubkey {
    Pubkey::from_base58(random_tip_account()).unwrap()
}

pub fn create_jito_tip_instruction_json(from: &str, lamports: u64) -> String {
    let tip_account = random_tip_account();

    let mut data = Vec::with_capacity(12);
    data.extend_from_slice(&2u32.to_le_bytes());
    data.extend_from_slice(&lamports.to_le_bytes());

    let instruction = SolanaInstruction {
        program_id: SYSTEM_PROGRAM_ID.to_string(),
        accounts: vec![
            SolanaAccountMeta {
                pubkey: from.to_string(),
                is_signer: true,
                is_writable: true,
            },
            SolanaAccountMeta {
                pubkey: tip_account.to_string(),
                is_signer: false,
                is_writable: true,
            },
        ],
        data: bs58::encode(&data).into_string(),
    };

    serde_json::to_string(&instruction).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_jito_tip_instruction_json() {
        let from = "7g2rVN8fAAQdPh1mkajpvELqYa3gWvFXJsBLnKfEQfqy";
        let lamports = 10_000u64;

        let json = create_jito_tip_instruction_json(from, lamports);
        let instruction: SolanaInstruction = serde_json::from_str(&json).unwrap();

        assert_eq!(instruction.program_id, SYSTEM_PROGRAM_ID);
        assert_eq!(instruction.accounts.len(), 2);
        assert_eq!(instruction.accounts[0].pubkey, from);
        assert!(instruction.accounts[0].is_signer);
        assert!(instruction.accounts[0].is_writable);
        assert!(JITO_TIP_ACCOUNTS.contains(&instruction.accounts[1].pubkey.as_str()));
        assert!(!instruction.accounts[1].is_signer);
        assert!(instruction.accounts[1].is_writable);

        let data = bs58::decode(&instruction.data).into_vec().unwrap();
        assert_eq!(data.len(), 12);

        let instruction_index = u32::from_le_bytes(data[0..4].try_into().unwrap());
        assert_eq!(instruction_index, 2);

        let decoded_lamports = u64::from_le_bytes(data[4..12].try_into().unwrap());
        assert_eq!(decoded_lamports, lamports);
    }
}
