use alloy_sol_types::sol;

sol! {
    #[derive(Debug, PartialEq)]
    interface IMonadStaking {
        function delegate(uint64 validatorId) external payable returns (bool success);
        function undelegate(uint64 validatorId, uint256 amount, uint8 withdrawId) external returns (bool success);
        function withdraw(uint64 validatorId, uint8 withdrawId) external returns (bool success);
        function claimRewards(uint64 validatorId) external returns (bool success);

        function getConsensusValidatorSet(uint32 startIndex) external view returns (bool isDone, uint32 nextIndex, uint64[] memory valIds);
        function getWithdrawalRequest(uint64 validatorId, address delegator, uint8 withdrawId)
            external
            view
            returns (uint256 withdrawalAmount, uint256 accRewardPerToken, uint64 withdrawEpoch);
        function getEpoch() external view returns (uint64 epoch, bool inEpochDelayPeriod);

        function getValidator(uint64 validatorId)
            external
            view
            returns (
                address authAddress,
                uint64 flags,
                uint256 stake,
                uint256 accRewardPerToken,
                uint256 commission,
                uint256 unclaimedRewards,
                uint256 consensusStake,
                uint256 consensusCommission,
                uint256 snapshotStake,
                uint256 snapshotCommission,
                bytes memory secpPubkey,
                bytes memory blsPubkey
            );

        function getDelegations(address delegator, uint64 startValId) external view returns (bool isDone, uint64 nextValId, uint64[] memory valIds);

        function getDelegator(uint64 validatorId, address delegator)
            external
            view
            returns (uint256 stake, uint256 accRewardPerToken, uint256 unclaimedRewards, uint256 deltaStake, uint256 nextDeltaStake, uint64 deltaEpoch, uint64 nextDeltaEpoch);
    }
}
