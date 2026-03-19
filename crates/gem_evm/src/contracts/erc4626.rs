use alloy_sol_types::sol;

sol! {
    interface IERC4626 {
        function balanceOf(address account) external view returns (uint256);
        function totalAssets() external view returns (uint256);
        function totalSupply() external view returns (uint256);
    }
}
