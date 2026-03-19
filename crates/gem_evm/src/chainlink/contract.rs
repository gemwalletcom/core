use alloy_sol_types::sol;
pub use primitives::contract_constants::{ETHEREUM_CHAINLINK_ETH_USD_FEED_CONTRACT, MONAD_CHAINLINK_USD_FEED_CONTRACT};

// https://github.com/smartcontractkit/chainlink/blob/develop/contracts/src/v0.8/shared/interfaces/AggregatorInterface.sol
sol! {
    interface AggregatorInterface {
        function latestRoundData() external view returns (uint80 roundId, int256 answer, uint256 startedAt, uint256 updatedAt, uint80 answeredInRound);
    }
}
