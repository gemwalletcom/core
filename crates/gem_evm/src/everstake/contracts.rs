use alloy_sol_types::sol;

sol! {
    #[derive(Debug, PartialEq)]
    interface IPool {
        function stake(uint64 source) payable returns (uint256);
        function unstake(uint256 value, uint16 allowedInterchangeNum, uint64 source) returns (uint256);
        function unstakePending(uint256 amount) returns (uint256);
    }

    #[derive(Debug, PartialEq)]
    interface IAccounting {
        function depositedBalanceOf(address account) view returns (uint256);
        function pendingBalanceOf(address account) view returns (uint256);
        function pendingDepositedBalanceOf(address account) view returns (uint256);
        function withdrawRequest(address staker) view returns (WithdrawRequest memory);
        function claimWithdrawRequest() external;
    }

    #[derive(Debug, PartialEq)]
    struct WithdrawRequest {
        uint256 requested;
        uint256 readyForClaim;
    }
}
