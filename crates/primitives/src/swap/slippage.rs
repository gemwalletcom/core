#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Slippage {
    pub bps: u32,
    pub mode: SlippageMode,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum SlippageMode {
    Auto,
    Exact,
}

impl From<u32> for Slippage {
    fn from(value: u32) -> Self {
        Slippage {
            bps: value,
            mode: SlippageMode::Exact,
        }
    }
}
