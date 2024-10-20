use alloy_core::sol;

// https://github.com/Uniswap/universal-router/blob/main/contracts/interfaces/IUniversalRouter.sol
sol! {
    /// @notice Executes encoded commands along with provided inputs. Reverts if deadline has expired.
    /// @param commands A set of concatenated commands, each 1 byte in length
    /// @param inputs An array of byte strings containing abi encoded inputs for each command
    /// @param deadline The deadline by which the transaction must be executed
    #[derive(Debug, PartialEq)]
    interface IUniversalRouter {
        function execute(bytes calldata commands, bytes[] calldata inputs, uint256 deadline) external payable;
    }
}

// https://github.com/Uniswap/v3-periphery/blob/main/contracts/interfaces/IQuoterV2.sol
sol! {
    #[derive(Debug, PartialEq)]
    struct QuoteExactInputSingleParams {
        address tokenIn;
        address tokenOut;
        uint256 amountIn;
        uint24 fee;
        uint160 sqrtPriceLimitX96;
    }

    /// @title QuoterV2 Interface
    /// @notice Supports quoting the calculated amounts from exact input or exact output swaps.
    /// @notice For each pool also tells you the number of initialized ticks crossed and the sqrt price of the pool after the swap.
    /// @dev These functions are not marked view because they rely on calling non-view functions and reverting
    /// to compute the result. They are also not gas efficient and should not be called on-chain.
    #[derive(Debug, PartialEq)]
    interface IQuoterV2 {
        /// @notice Returns the amount out received for a given exact input swap without executing the swap
        /// @param path The path of the swap, i.e. each token pair and the pool fee
        /// @param amountIn The amount of the first token to swap
        /// @return amountOut The amount of the last token that would be received
        /// @return sqrtPriceX96AfterList List of the sqrt price after the swap for each pool in the path
        /// @return initializedTicksCrossedList List of the initialized ticks that the swap crossed for each pool in the path
        /// @return gasEstimate The estimate of the gas that the swap consumes
        function quoteExactInput(bytes memory path, uint256 amountIn)
        external
        returns (
            uint256 amountOut,
            uint160[] memory sqrtPriceX96AfterList,
            uint32[] memory initializedTicksCrossedList,
            uint256 gasEstimate
        );

        /// @notice Returns the amount out received for a given exact input but for a swap of a single pool
        /// @param params The params for the quote, encoded as `QuoteExactInputSingleParams`
        /// tokenIn The token being swapped in
        /// tokenOut The token being swapped out
        /// fee The fee of the token pool to consider for the pair
        /// amountIn The desired input amount
        /// sqrtPriceLimitX96 The price limit of the pool that cannot be exceeded by the swap
        /// @return amountOut The amount of `tokenOut` that would be received
        /// @return sqrtPriceX96After The sqrt price of the pool after the swap
        /// @return initializedTicksCrossed The number of initialized ticks that the swap crossed
        /// @return gasEstimate The estimate of the gas that the swap consumes
        function quoteExactInputSingle(QuoteExactInputSingleParams memory params)
        external
        returns (
            uint256 amountOut,
            uint160 sqrtPriceX96After,
            uint32 initializedTicksCrossed,
            uint256 gasEstimate
        );
    }
}
