use super::WHIRLPOOL_CONFIG;
use base64::{engine::general_purpose::STANDARD, Engine};
use borsh::{BorshDeserialize, BorshSerialize};
use gem_solana::pubkey::Pubkey;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

pub const NUM_REWARDS: usize = 3;
pub const TICK_ARRAY_SIZE: usize = 88;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgramAccount {
    pub account: AccountData,
    pub pubkey: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountData {
    pub data: Vec<String>,
    pub owner: String,
    pub space: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValueResult<T> {
    pub value: T,
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct FeeTier {
    pub discriminator: [u8; 8],
    pub whirlpools_config: Pubkey,
    pub tick_spacing: u16,
    pub default_fee_rate: u16,
}

impl FeeTier {
    pub fn new(tick_spacing: u16, default_fee_rate: u16) -> Self {
        Self {
            discriminator: [0; 8],
            whirlpools_config: Pubkey::from_str(WHIRLPOOL_CONFIG).unwrap(),
            tick_spacing,
            default_fee_rate,
        }
    }
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct WhirlpoolsConfig {
    pub discriminator: [u8; 8],
    pub fee_authority: Pubkey,
    pub collect_protocol_fees_authority: Pubkey,
    pub reward_emissions_super_authority: Pubkey,
    pub default_protocol_fee_rate: u16,
    pub config_reserved: [u8; 2],
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct Whirlpool {
    pub discriminator: [u8; 8],
    pub whirlpools_config: Pubkey, // 32
    pub whirlpool_bump: [u8; 1],   // 1

    pub tick_spacing: u16,          // 2
    pub tick_spacing_seed: [u8; 2], // 2

    // Stored as hundredths of a basis point
    // u16::MAX corresponds to ~6.5%
    pub fee_rate: u16, // 2

    // Portion of fee rate taken stored as basis points
    pub protocol_fee_rate: u16, // 2

    // Maximum amount that can be held by Solana account
    pub liquidity: u128, // 16

    // MAX/MIN at Q32.64, but using Q64.64 for rounder bytes
    // Q64.64
    pub sqrt_price: u128,        // 16
    pub tick_current_index: i32, // 4

    pub protocol_fee_owed_a: u64, // 8
    pub protocol_fee_owed_b: u64, // 8

    pub token_mint_a: Pubkey,  // 32
    pub token_vault_a: Pubkey, // 32

    // Q64.64
    pub fee_growth_global_a: u128, // 16

    pub token_mint_b: Pubkey,  // 32
    pub token_vault_b: Pubkey, // 32

    // Q64.64
    pub fee_growth_global_b: u128, // 16

    pub reward_last_updated_timestamp: u64, // 8

    pub reward_infos: [WhirlpoolRewardInfo; NUM_REWARDS], // 384
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct WhirlpoolRewardInfo {
    /// Reward token mint.
    pub mint: Pubkey,
    /// Reward vault token account.
    pub vault: Pubkey,
    /// Authority account that has permission to initialize the reward and set emissions.
    pub authority: Pubkey,
    /// Q64.64 number that indicates how many tokens per second are earned per unit of liquidity.
    pub emissions_per_second_x64: u128,
    /// Q64.64 number that tracks the total tokens earned per unit of liquidity since the reward
    /// emissions were turned on.
    pub growth_global_x64: u128,
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct Tick {
    pub initialized: bool,
    pub liquidity_net: i128,
    pub liquidity_gross: u128,
    pub fee_growth_outside_a: u128,
    pub fee_growth_outside_b: u128,
    pub reward_growths_outside: [u128; 3],
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct TickArray {
    pub discriminator: [u8; 8],
    pub start_tick_index: i32,
    pub ticks: [Tick; TICK_ARRAY_SIZE],
    pub whirlpool: Pubkey,
}

pub fn try_borsh_decode<T: BorshDeserialize>(base64_str: &str) -> Result<T, anyhow::Error> {
    let bytes = STANDARD.decode(base64_str)?;
    T::try_from_slice(&bytes).map_err(|e| anyhow::anyhow!("borsh deserialize error: {:?}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_fee_tier() {
        let base64_str = "OEufTI5EvmkT5EH4ORPKaLBjT7Al/eqohzfoQRDRJV41ezN33e4czQgA9AE=";
        let fee_tier: FeeTier = try_borsh_decode(base64_str).unwrap();

        assert_eq!(fee_tier.tick_spacing, 8);
        assert_eq!(fee_tier.default_fee_rate, 500);

        let base64_str = "OEufTI5EvmkT5EH4ORPKaLBjT7Al/eqohzfoQRDRJV41ezN33e4czRAAQAY=";
        let fee_tier: FeeTier = try_borsh_decode(base64_str).unwrap();

        assert_eq!(fee_tier.tick_spacing, 16);
        assert_eq!(fee_tier.default_fee_rate, 1600);

        let base64_str = "OEufTI5EvmkT5EH4ORPKaLBjT7Al/eqohzfoQRDRJV41ezN33e4czYCAECc=";
        let fee_tier: FeeTier = try_borsh_decode(base64_str).unwrap();

        assert_eq!(fee_tier.tick_spacing, 32896);
        assert_eq!(fee_tier.default_fee_rate, 10000);
    }

    #[test]
    fn test_decode_whirlpools_config() {
        let base64_str =
            "nRQx4NlXwf5M8EAKh2YmU2LwaOq5bUcaRw0MLf22JZCC2KJOfKDOCIsXeapny4zlIwOcpuBqx7UNjpTOIDGrVG0ThyZLCQKwvR0xrxfe/zwmhIFgCsr+SxQJjA/hQbf0oc34STRkRAMsAQAA";
        let config: WhirlpoolsConfig = try_borsh_decode(base64_str).unwrap();

        assert_eq!(config.fee_authority.to_string(), "6BLTtBS9miUZruZtR9reTzp6ctGc4kVY4xrcxQwurYtw");
        assert_eq!(
            config.collect_protocol_fees_authority.to_string(),
            "AMxS2z98KhS3ZJAtMG7PkapJvP6sJTCMhmcFPZiUovMu"
        );
        assert_eq!(
            config.reward_emissions_super_authority.to_string(),
            "DjDsi34mSB66p2nhBL6YvhbcLtZbkGfNybFeLDjJqxJW"
        );
        assert_eq!(config.default_protocol_fee_rate, 300);
    }

    #[test]
    fn test_decode_whirlpool() {
        let base64_str = "P5XRDOGAYwkT5EH4ORPKaLBjT7Al/eqohzfoQRDRJV41ezN33e4czf4BAAEAZAABAPidqZ5fIwAAAAAAAAAAAACjuAdX+Trn/wAAAAAAAAAA+P///1O/HAAAAAAA3y8cAAAAAAAXkkg7bIoqh7dHHYFPlZH5OVyECpzj2fTVun06S4p0nmUsaIGkTdFrz+MYnhYfOgyz4zQ1cmq2FBWRoO4tzuiUY92FEEzxAQAAAAAAAAAAAM4BDmCv7bInF71jGS9UFFo/llozu4LSxwKess4eIIJkPy5AymtnaszpL/DB7lvH9eripWxKGREM6eGZTxFvw4LsxIIJHPEBAAAAAAAAAAAAdmIsZwAAAAAXkkg7bIoqh7dHHYFPlZH5OVyECpzj2fTVun06S4p0niZgmq1j03IXXG1fM6HZvXY2IXujofTSqQbIBaFajuR/vR0xrxfe/zwmhIFgCsr+SxQJjA/hQbf0oc34STRkRAMAAAAAAAAAAAAAAAAAAAAA1UwtgntDGQAAAAAAAAAAAAwA0K/rhhTafxmroC1A8YxpJYX2UCDfztPV5fmpwMThA8hYP14GHa5nRacL5ZcsWCGrc2hIYoozCeT8FCU7/Ru9HTGvF97/PCaEgWAKyv5LFAmMD+FBt/ShzfhJNGREAwAAAAAAAAAAAAAAAAAAAAC80iMq3qADAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAL0dMa8X3v88JoSBYArK/ksUCYwP4UG39KHN+Ek0ZEQDAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";
        let whirlpool: Whirlpool = try_borsh_decode(base64_str).unwrap();

        assert_eq!(whirlpool.whirlpools_config.to_string(), "2LecshUwdy9xi7meFgHtFJQNSKk4KdTrcpvaB56dP2NQ");
        assert_eq!(whirlpool.tick_spacing, 1);
        assert_eq!(whirlpool.fee_rate, 100);
        assert_eq!(whirlpool.liquidity, 38893590781432);
        assert_eq!(whirlpool.token_mint_a.to_string(), "2b1kV6DkPAnxd5ixfnxCpjxmKwqjjaYmCZfHsFu24GXo");
        assert_eq!(whirlpool.token_mint_b.to_string(), "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB");
        assert_eq!(whirlpool.reward_infos[0].mint.to_string(), "2b1kV6DkPAnxd5ixfnxCpjxmKwqjjaYmCZfHsFu24GXo");
        assert_eq!(whirlpool.reward_infos[1].mint.to_string(), "orcaEKTdK7LKz57vaAYr9QeNsVEPfiu6QeMU1kektZE");
    }
}
