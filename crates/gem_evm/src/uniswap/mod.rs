pub mod command;
pub mod contract;
pub mod deployment;

// hundredths of bps (e.g. 0.3% is 3000)
#[derive(Debug, Clone, PartialEq)]
pub enum FeeTier {
    Lowest = 100,
    Low = 500,
    Medium = 3000,
    High = 10000,
}

impl TryFrom<&str> for FeeTier {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "100" => Ok(FeeTier::Lowest),
            "500" => Ok(FeeTier::Low),
            "3000" => Ok(FeeTier::Medium),
            "10000" => Ok(FeeTier::High),
            _ => Err(anyhow::anyhow!("invalid fee tier: {}", value)),
        }
    }
}
