use alloy_sol_types::sol;

// https://github.com/smartcontractkit/chainlink/blob/develop/contracts/src/v0.8/shared/interfaces/AggregatorInterface.sol
sol! {
    interface AggregatorInterface {
        function latestRoundData() external view returns (uint80 roundId, int256 answer, uint256 startedAt, uint256 updatedAt, uint80 answeredInRound);
    }
}

pub const CHAINLINK_ETH_USD_FEED: &str = "0x5f4eC3Df9cbd43714FE2740f5E3616155c5b8419";
pub const CHAINLINK_SOL_USD_FEED: &str = "0x4ffC43a60e009B551865A93d232E33Fce9f01507";
