use alloy_core::sol;

// https://docs.across.to/reference/selected-contract-functions
// https://github.com/across-protocol/contracts/blob/master/contracts/HubPool.sol
sol! {
    interface HubPoolInterface {
        function paused() external view returns (bool);
    }
}
