use alloy_core::sol;

sol! {
    interface IERC20 {
        function allowance(address owner, address spender) external view returns (uint256);
        function transfer(address to, uint256 value) external returns (bool);
        function decimals() external view returns (uint8);
    }
}
