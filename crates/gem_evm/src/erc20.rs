use alloy_sol_types::sol;

sol! {
    interface IERC20 {
        function name() public view virtual returns (string memory);
        function symbol() public view virtual returns (string memory);
        function decimals() public view virtual returns (uint8);

        function allowance(address owner, address spender) external view returns (uint256);
        function transfer(address to, uint256 value) external returns (bool);
    }
}
