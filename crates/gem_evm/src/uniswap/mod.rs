pub mod command;
pub mod contract;
pub mod deployment;
pub mod path;

// hundredths of bps (e.g. 0.3% is 3000)
#[derive(Debug, Clone, PartialEq)]
pub enum FeeTier {
    Hundred = 100,
    FiveHundred = 500,
    TwoThousandFiveHundred = 2500,
    ThreeThousand = 3000,
    TenThousand = 10000,
}

impl TryFrom<&str> for FeeTier {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "100" => Ok(FeeTier::Hundred),
            "500" => Ok(FeeTier::FiveHundred),
            "2500" => Ok(FeeTier::TwoThousandFiveHundred),
            "3000" => Ok(FeeTier::ThreeThousand),
            "10000" => Ok(FeeTier::TenThousand),
            _ => Err(anyhow::anyhow!("invalid fee tier: {}", value)),
        }
    }
}
