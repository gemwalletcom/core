use alloy_core::sol;

sol!(
    function name() public view virtual returns (string memory);
    function symbol() public view virtual returns (string memory);
    function decimals() public view virtual returns (uint8);
);
