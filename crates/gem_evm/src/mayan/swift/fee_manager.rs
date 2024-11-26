use alloy_core::sol;
use alloy_primitives::{Address, U256};

sol! {
    /// @notice Fee Manager interface for managing protocol fees and treasury operations
    #[derive(Debug, PartialEq)]
    interface IFeeManager {
        /// @notice Calculates the protocol fee in basis points
        /// @param amountIn The input amount for the swap
        /// @param tokenIn The input token address
        /// @param tokenOut The output token identifier
        /// @param destChain The destination chain identifier
        /// @param referrerBps The referrer's basis points
        /// @return The protocol fee in basis points
        function calcProtocolBps(
            uint64 amountIn,
            address tokenIn,
            bytes32 tokenOut,
            uint16 destChain,
            uint8 referrerBps
        ) external view returns (uint8);

        /// @notice Returns the current fee collector address
        /// @return The address of the fee collector (treasury or contract)
        function feeCollector() external view returns (address);

        /// @notice Changes the operator to a new address
        /// @param nextOperator The address of the new operator
        function changeOperator(address nextOperator) external;

        /// @notice Allows the next operator to claim the operator role
        function claimOperator() external;

        /// @notice Sweeps ERC20 tokens from the contract
        /// @param token The token address to sweep
        /// @param amount The amount to sweep
        /// @param to The recipient address
        function sweepToken(address token, uint256 amount, address to) external;

        /// @notice Sweeps ETH from the contract
        /// @param amount The amount of ETH to sweep
        /// @param to The recipient address
        function sweepEth(uint256 amount, address payable to) external;

        /// @notice Sets the base fee in basis points
        /// @param baseBps The new base fee in basis points
        function setBaseBps(uint8 baseBps) external;

        /// @notice Sets the treasury address
        /// @param treasury The new treasury address
        function setTreasury(address treasury) external;
    }

    /// @notice Fee Manager contract state and events
    #[derive(Debug, PartialEq)]
    contract FeeManager {
        /// @notice The current operator address
        address public operator;

        /// @notice The next operator address
        address public nextOperator;

        /// @notice The base fee in basis points
        uint8 public baseBps;

        /// @notice The treasury address
        address public treasury;
    }
}
