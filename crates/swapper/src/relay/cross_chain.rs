use primitives::Transaction;

use crate::SwapperProvider;
use crate::cross_chain::CrossChainProvider;

// https://docs.relay.link/references/api/api_resources/contract-addresses
pub const RELAY_CONTRACTS: [&str; 5] = [
    "0x4cD00E387622C35bDDB9b4c962C136462338BC31",  // Depository
    "0x59916DA825D2D2eC1BF878D71c88826F6633ecca",  // Depository (alt)
    "0xf70da97812CB96acDF810712Aa562db8dfA3dbEF",  // Solver EVM
    "bc1qq2mvrp4g3ugd424dw4xv53rgsf8szkrv853jrc", // Solver Bitcoin
    "F7p3dFrjRTbtRp8FRF6qHLomXbKRBzpvBLjtQcfcgmNe", // Solver Solana
];

pub struct RelayCrossChain;

impl CrossChainProvider for RelayCrossChain {
    fn provider(&self) -> SwapperProvider {
        SwapperProvider::Relay
    }

    fn is_swap(&self, transaction: &Transaction) -> bool {
        RELAY_CONTRACTS.iter().any(|x| transaction.to.eq_ignore_ascii_case(x) || transaction.from.eq_ignore_ascii_case(x))
    }
}
