use alloy_sol_types::sol;

sol! {
    interface IERC20 {
        function allowance(address owner, address spender) external view returns (uint256);
        function transfer(address to, uint256 value) external returns (bool);
    }
}
