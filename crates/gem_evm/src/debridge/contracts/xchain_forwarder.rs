use alloy_core::sol;

// https://etherscan.io/address/0x663dc15d3c1ac63ff12e45ab68fea3f0a883c251#readProxyContract
sol! {
    interface CrosschainForwarder {
        /// @dev Performs swap against arbitrary input token, refunds excessive outcome of such swap (if any),
        ///      and calls the specified receiver supplying the outcome of the swap
        /// @param _srcTokenIn arbitrary input token to swap from
        /// @param _srcAmountIn amount of input token to swap
        /// @param _srcTokenInPermitEnvelope optional permit envelope to grab the token from the caller. bytes (amount + deadline + signature)
        /// @param _srcSwapRouter contract to call that performs swap from the input token to the output token
        /// @param _srcSwapCalldata calldata to call against _srcSwapRouter
        /// @param _srcTokenOut arbitrary output token to swap to
        /// @param _srcTokenExpectedAmountOut minimum acceptable outcome of the swap to provide to _target
        /// @param _srcTokenRefundRecipient address to send excessive outcome of the swap
        /// @param _target contract to call after successful swap
        /// @param _targetData calldata to call against _target
        function strictlySwapAndCall(
            address _srcTokenIn,
            uint256 _srcAmountIn,
            bytes memory _srcTokenInPermitEnvelope,
            address _srcSwapRouter,
            bytes calldata _srcSwapCalldata,

            address _srcTokenOut,
            uint _srcTokenExpectedAmountOut,
            address _srcTokenRefundRecipient,

            address _target,
            bytes calldata _targetData
        ) external payable;
    }
}
