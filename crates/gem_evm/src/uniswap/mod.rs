pub mod actions;
pub mod command;
pub mod contracts;
pub mod deployment;
pub mod path;

// hundredths of bps (e.g. 0.3% is 3000)
#[derive(Debug, Clone, PartialEq)]
#[repr(u32)]
pub enum FeeTier {
    Hundred = 100,
    FiveHundred = 500,
    TwoThousandFiveHundred = 2500,
    ThreeThousand = 3000,
    TenThousand = 10000,
}

impl FeeTier {
    pub fn default_tick_spacing(&self) -> u32 {
        match self {
            FeeTier::Hundred => 1,
            FeeTier::FiveHundred => 10,
            FeeTier::TwoThousandFiveHundred => 50,
            FeeTier::ThreeThousand => 60,
            FeeTier::TenThousand => 200,
        }
    }
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
