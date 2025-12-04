use alloy_sol_types::sol;

sol! {
    #[derive(Debug, PartialEq)]
    interface IMonadStaking {
        function delegate(uint64 validatorId) external payable returns (bool success);
        function undelegate(uint64 validatorId, uint256 amount, uint8 withdrawId) external returns (bool success);
        function withdraw(uint64 validatorId, uint8 withdrawId) external returns (bool success);
        function claimRewards(uint64 validatorId) external returns (bool success);

    }
}

sol! {
    #[derive(Debug, PartialEq)]
    interface IMonadStakingLens {
        enum DelegationState {
            Active,
            Activating,
            Deactivating,
            AwaitingWithdrawal
        }

        struct DelegationPosition {
            uint64 validatorId;
            uint8 withdrawId;
            DelegationState state;
            uint256 amount;
            uint256 rewards;
            uint64 withdrawEpoch;
            uint64 completionTimestamp;
        }

        struct ValidatorInfo {
            uint64 validatorId;
            uint256 stake;
            uint256 commission;
            uint64 apyBps;
            bool isActive;
        }

        function getBalance(address delegator) external returns (uint256 staked, uint256 pending, uint256 rewards);

        function getDelegations(address delegator) external returns (DelegationPosition[] memory positions);

        function getValidators(uint64[] calldata validatorIds) external returns (ValidatorInfo[] memory validators, uint64 networkApyBps);

        function getAPYs(uint64[] calldata validatorIds) external returns (uint64[] memory apysBps);
    }
}
