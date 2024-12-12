use alloy_core::sol;

// https://docs.across.to/reference/selected-contract-functions
// https://github.com/across-protocol/contracts/blob/master/contracts/HubPool.sol
sol! {
    interface HubPoolInterface {
        // Each whitelisted L1 token has an associated pooledToken struct that contains all information used to track the
        // cumulative LP positions and if this token is enabled for deposits.
        struct PooledToken {
            // LP token given to LPs of a specific L1 token.
            address lpToken;
            // True if accepting new LP's.
            bool isEnabled;
            // Timestamp of last LP fee update.
            uint32 lastLpFeeUpdate;
            // Number of LP funds sent via pool rebalances to SpokePools and are expected to be sent
            // back later.
            int256 utilizedReserves;
            // Number of LP funds held in contract less utilized reserves.
            uint256 liquidReserves;
            // Number of LP funds reserved to pay out to LPs as fees.
            uint256 undistributedLpFees;
        }

        function paused() external view returns (bool);
        function liquidityUtilizationCurrent(address l1Token) external returns (uint256);
        function liquidityUtilizationPostRelay(address l1Token, uint256 relayedAmount) external returns (uint256);
    }
}

// cast call 0xc186fA914353c44b2E33eBE05f21846F1048bEda "liquidityUtilizationCurrent(address)(uint256)" 0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2 --rpc-url
