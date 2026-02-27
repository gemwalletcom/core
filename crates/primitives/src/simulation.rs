use crate::BalanceDiffMap;

#[derive(Debug, Clone)]
pub struct SimulationInput {
    pub encoded_transaction: String,
}

#[derive(Debug, Clone)]
pub struct SimulationResult {
    pub success: bool,
    pub error: Option<String>,
    pub logs: Vec<String>,
    pub units_consumed: Option<u64>,
    pub balance_changes: BalanceDiffMap,
}
