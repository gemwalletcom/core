use primitives::SwapQuoteEthereumData;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteRequest {
    pub src: String,
    pub dst: String,
    pub from: String,
    pub amount: String,
    pub slippage: f64,
    pub disable_estimate: bool,
    pub fee: f64,
    pub referrer: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Allowance {
    pub allowance: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapResult {
    pub tx: SwapResultTransaction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapResultTransaction {
    pub to: String,
    pub value: String,
    pub data: String,
}

impl SwapResultTransaction {
    pub fn get_data(&self) -> SwapQuoteEthereumData {
        SwapQuoteEthereumData{
            to: self.to.clone(),
            value: self.value.clone(),
            data: self.data.clone(),
        }
    }
}