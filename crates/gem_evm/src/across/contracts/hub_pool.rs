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
    }
}
