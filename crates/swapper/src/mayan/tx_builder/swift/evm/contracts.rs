use alloy_sol_types::sol;

sol! {
    interface MayanForwarder {
        #[derive(Default)]
        struct PermitParams {
            uint256 value;
            uint256 deadline;
            uint8 v;
            bytes32 r;
            bytes32 s;
        }

        function forwardERC20(address tokenIn, uint256 amountIn, PermitParams permitParams, address mayanProtocol, bytes protocolData) external payable;
        function swapAndForwardERC20(
            address tokenIn,
            uint256 amountIn,
            PermitParams permitParams,
            address swapProtocol,
            bytes swapData,
            address middleToken,
            uint256 minMiddleAmount,
            address mayanProtocol,
            bytes mayanData
        ) external payable;
        function swapAndForwardEth(
            uint256 amountIn,
            address swapProtocol,
            bytes swapData,
            address middleToken,
            uint256 minMiddleAmount,
            address mayanProtocol,
            bytes mayanData
        ) external payable;
    }

    interface MayanSwiftV2 {
        struct OrderParams {
            uint8 payloadType;
            bytes32 trader;
            bytes32 destAddr;
            uint16 destChainId;
            bytes32 referrerAddr;
            bytes32 tokenOut;
            uint64 minAmountOut;
            uint64 gasDrop;
            uint64 cancelFee;
            uint64 refundFee;
            uint64 deadline;
            uint8 referrerBps;
            uint8 auctionMode;
            bytes32 random;
        }

        function createOrderWithToken(address tokenIn, uint256 amountIn, OrderParams params, bytes customPayload) external payable returns (bytes32 orderHash);
    }
}
