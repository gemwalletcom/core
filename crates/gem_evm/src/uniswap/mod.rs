pub mod command;
pub mod contract;
pub mod deployment;

// hundredths of bps (e.g. 0.3% is 3000)
#[derive(Debug, PartialEq)]
pub enum FeeTier {
    Lowest = 100,
    Low = 500,
    Medium = 3000,
    High = 10000,
}
