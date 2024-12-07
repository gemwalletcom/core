use alloy_core::sol;

sol! {
    /// @title MayanSwift Cross-Chain Swap Contract
    #[derive(Debug)]
    contract IMayanSwift{
        struct OrderParams {
            bytes32 trader;
            bytes32 tokenOut;
            uint64 minAmountOut;
            uint64 gasDrop;
            uint64 cancelFee;
            uint64 refundFee;
            uint64 deadline;
            bytes32 destAddr;
            uint16 destChainId;
            bytes32 referrerAddr;
            uint8 referrerBps;
            uint8 auctionMode;
            bytes32 random;
        }

        struct PermitParams {
            uint256 value;
            uint256 deadline;
            uint8 v;
            bytes32 r;
            bytes32 s;
        }

        function createOrderWithEth(OrderParams memory params) external payable returns (bytes32 orderHash);
        function createOrderWithToken(address tokenIn, uint256 amountIn, OrderParams memory params) external returns (bytes32 orderHash);
    }
}
