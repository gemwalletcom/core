use alloy_core::sol;

sol! {
    /// @title MayanForwarder Interface
    #[derive(Debug, PartialEq)]
    interface IMayanForwarder {
        /// @notice Guardian address
        function guardian() external view returns (address);

        /// @notice Next guardian address
        function nextGuardian() external view returns (address);

        /// @notice Check if protocol is supported for swaps
        function swapProtocols(address protocol) external view returns (bool);

        /// @notice Check if protocol is supported for Mayan operations
        function mayanProtocols(address protocol) external view returns (bool);

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

        /// @notice Rescue ERC20 tokens
        function rescueToken(address token, uint256 amount, address to) external;

        /// @notice Rescue ETH
        function rescueEth(uint256 amount, address payable to) external;

        /// @notice Change guardian
        function changeGuardian(address newGuardian) external;

        /// @notice Claim guardian role
        function claimGuardian() external;

        /// @notice Set swap protocol status
        function setSwapProtocol(address swapProtocol, bool enabled) external;

        /// @notice Set Mayan protocol status
        function setMayanProtocol(address mayanProtocol, bool enabled) external;

        /// Events
        event ForwardedEth(address mayanProtocol, bytes protocolData);
        event ForwardedERC20(address token, uint256 amount, address mayanProtocol, bytes protocolData);
        event SwapAndForwardedEth(uint256 amountIn, address swapProtocol, address middleToken, uint256 middleAmount, address mayanProtocol, bytes mayanData);
        event SwapAndForwardedERC20(address tokenIn, uint256 amountIn, address swapProtocol, address middleToken, uint256 middleAmount, address mayanProtocol, bytes mayanData);

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
