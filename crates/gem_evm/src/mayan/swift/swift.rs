use alloy_core::sol;

// First define the enums used in the contract
sol! {
    #[derive(Debug)]
    enum Status {
        CREATED,
        FULFILLED,
        UNLOCKED,
        CANCELED,
        REFUNDED
    }

    enum Action {
        NONE,
        FULFILL,
        UNLOCK,
        REFUND,
        BATCH_UNLOCK
    }

    enum AuctionMode {
        NONE,
        BYPASS,
        ENGLISH
    }
}

// Now define the main contract interface
sol! {
    /// @title MayanSwift Cross-Chain Swap Contract
    #[derive(Debug)]
    contract IMayanSwift{
        // Events
        event OrderCreated(bytes32 indexed key);
        event OrderFulfilled(bytes32 indexed key, uint64 sequence, uint256 netAmount);
        event OrderUnlocked(bytes32 indexed key);
        event OrderCanceled(bytes32 indexed key, uint64 sequence);
        event OrderRefunded(bytes32 indexed key, uint256 netAmount);

        // Storage
        address public immutable wormhole;
        uint16 public immutable auctionChainId;
        bytes32 public immutable auctionAddr;
        bytes32 public immutable solanaEmitter;
        address public feeManager;
        uint8 public consistencyLevel;
        address public guardian;
        address public nextGuardian;
        bool public paused;

        struct Order {
            uint8 status;  // Status enum
            uint64 amountIn;
            uint16 destChainId;
        }

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

        struct KeyStruct {
            OrderParams params;
            bytes32 tokenIn;
            uint16 chainId;
            uint16 protocolBps;
        }

        // State changing functions
        function setPause(bool _pause) external;
        function setFeeManager(address _feeManager) external;
        function setConsistencyLevel(uint8 _consistencyLevel) external;
        function changeGuardian(address newGuardian) external;
        function claimGuardian() external;

        // External functions
        function createOrderWithEth(OrderParams memory params) external payable returns (bytes32 orderHash);
        function createOrderWithToken(address tokenIn, uint256 amountIn, OrderParams memory params) external returns (bytes32 orderHash);
        function createOrderWithSig(
            address tokenIn,
            uint256 amountIn,
            OrderParams memory params,
            uint256 submissionFee,
            bytes calldata signedOrderHash,
            PermitParams calldata permitParams
        ) external returns (bytes32 orderHash);

        // View functions
        function getOrders(bytes32[] memory orderHashes) external view returns (Order[] memory);
    }
}
