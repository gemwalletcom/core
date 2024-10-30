use alloy_core::sol;

sol! {
    interface IERC20 {
        function allowance(address owner, address spender) external view returns (uint256);
    }
}
