use alloy_core::sol;

sol! {
    #[derive(Debug, PartialEq)]
    interface IUniversalRouter {
        function execute(bytes calldata commands, bytes[] calldata inputs, uint256 deadline) external payable;
    }
}
