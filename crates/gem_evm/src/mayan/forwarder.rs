use alloy_core::sol;

sol! {
    /// @title MayanForwarder Interface
    #[derive(Debug, PartialEq)]
    interface IMayanForwarder {

        /// @notice Forward ETH to Mayan protocol
        function forwardEth(address mayanProtocol, bytes calldata protocolData) external payable;

        /// @notice Forward ERC20 tokens to Mayan protocol
        function forwardERC20(
            address tokenIn,
            uint256 amountIn,
            PermitParams calldata permitParams,
            address mayanProtocol,
            bytes calldata protocolData
        ) external payable;

        /// @notice Swap ETH to token and forward to Mayan protocol
        function swapAndForwardEth(
            uint256 amountIn,
            address swapProtocol,
            bytes calldata swapData,
            address middleToken,
            uint256 minMiddleAmount,
            address mayanProtocol,
            bytes calldata mayanData
        ) external payable;

        /// @notice Swap ERC20 token and forward to Mayan protocol
        function swapAndForwardERC20(
            address tokenIn,
            uint256 amountIn,
            PermitParams calldata permitParams,
            address swapProtocol,
            bytes calldata swapData,
            address middleToken,
            uint256 minMiddleAmount,
            address mayanProtocol,
            bytes calldata mayanData
        ) external payable;

        /// Structs
        struct PermitParams {
            uint256 value;
            uint256 deadline;
            uint8 v;
            bytes32 r;
            bytes32 s;
        }
    }
}
