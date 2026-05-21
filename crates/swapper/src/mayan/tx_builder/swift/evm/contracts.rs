use alloy_sol_types::sol;

sol! {
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
