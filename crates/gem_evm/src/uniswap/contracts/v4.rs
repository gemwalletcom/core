use alloy_core::sol;

// https://github.com/Uniswap/v4-periphery/blob/main/src/interfaces/IV4Quoter.sol
sol! {
    type Currency is address;

    #[derive(Debug)]
    struct PoolKey {
        /// @notice The lower currency of the pool, sorted numerically
        Currency currency0;
        /// @notice The higher currency of the pool, sorted numerically
        Currency currency1;
        /// @notice The pool LP fee, capped at 1_000_000. If the highest bit is 1, the pool has a dynamic fee and must be exactly equal to 0x800000
        uint24 fee;
        /// @notice Ticks that involve positions must be a multiple of tick spacing
        int24 tickSpacing;
        /// @notice The hooks of the pool
        address hooks;
    }

    #[derive(Debug)]
    struct PathKey {
        Currency intermediateCurrency;
        uint24 fee;
        int24 tickSpacing;
        address hooks;
        bytes hookData;
    }

    #[derive(Debug)]
    interface IV4Quoter {
        struct QuoteExactSingleParams {
            PoolKey poolKey;
            bool zeroForOne;
            uint128 exactAmount;
            bytes hookData;
        }

        struct QuoteExactParams {
            Currency exactCurrency;
            PathKey[] path;
            uint128 exactAmount;
        }

        /// @notice Returns the delta amounts for a given exact input swap of a single pool
        /// @param params The params for the quote, encoded as `QuoteExactSingleParams`
        /// poolKey The key for identifying a V4 pool
        /// zeroForOne If the swap is from currency0 to currency1
        /// exactAmount The desired input amount
        /// hookData arbitrary hookData to pass into the associated hooks
        /// @return amountOut The output quote for the exactIn swap
        /// @return gasEstimate Estimated gas units used for the swap
        function quoteExactInputSingle(QuoteExactSingleParams memory params)
            external
            returns (uint256 amountOut, uint256 gasEstimate);

        /// @notice Returns the delta amounts along the swap path for a given exact input swap
        /// @param params the params for the quote, encoded as 'QuoteExactParams'
        /// currencyIn The input currency of the swap
        /// path The path of the swap encoded as PathKeys that contains currency, fee, tickSpacing, and hook info
        /// exactAmount The desired input amount
        /// @return amountOut The output quote for the exactIn swap
        /// @return gasEstimate Estimated gas units used for the swap
        function quoteExactInput(QuoteExactParams memory params)
            external
            returns (uint256 amountOut, uint256 gasEstimate);

        /// @notice Returns the delta amounts for a given exact output swap of a single pool
        /// @param params The params for the quote, encoded as `QuoteExactSingleParams`
        /// poolKey The key for identifying a V4 pool
        /// zeroForOne If the swap is from currency0 to currency1
        /// exactAmount The desired output amount
        /// hookData arbitrary hookData to pass into the associated hooks
        /// @return amountIn The input quote for the exactOut swap
        /// @return gasEstimate Estimated gas units used for the swap
        function quoteExactOutputSingle(QuoteExactSingleParams memory params)
            external
            returns (uint256 amountIn, uint256 gasEstimate);

        /// @notice Returns the delta amounts along the swap path for a given exact output swap
        /// @param params the params for the quote, encoded as 'QuoteExactParams'
        /// currencyOut The output currency of the swap
        /// path The path of the swap encoded as PathKeys that contains currency, fee, tickSpacing, and hook info
        /// exactAmount The desired output amount
        /// @return amountIn The input quote for the exactOut swap
        /// @return gasEstimate Estimated gas units used for the swap
        function quoteExactOutput(QuoteExactParams memory params)
            external
            returns (uint256 amountIn, uint256 gasEstimate);
    }
}
