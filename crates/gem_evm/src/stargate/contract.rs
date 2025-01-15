use alloy_core::sol;
use serde::{Deserialize, Serialize};

sol! {
    /**
     * @dev Struct representing token parameters for the OFT send() operation.
     */
    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct SendParam {
        uint32  dstEid;        // Destination endpoint ID.
        bytes32 to;            // Recipient address.
        uint256 amountLD;      // Amount to send in local decimals.
        uint256 minAmountLD;   // Minimum amount to send in local decimals.
        bytes   extraOptions;  // Additional options supplied by the caller to be used in the LayerZero message.
        bytes   composeMsg;    // The composed message for the send() operation.
        bytes   oftCmd;        // The OFT command to be executed, unused in default OFT implementations.
    }

    /**
     * @dev Struct representing OFT limit information.
     * @dev These amounts can change dynamically and are up the the specific oft implementation.
     */
    #[derive(Debug, PartialEq)]
    struct OFTLimit {
        uint256 minAmountLD;   // Minimum amount in local decimals that can be sent to the recipient.
        uint256 maxAmountLD;   // Maximum amount in local decimals that can be sent to the recipient.
    }

    /**
     * @dev Struct representing OFT receipt information.
     */
    #[derive(Debug, PartialEq)]
    struct OFTReceipt {
        uint256 amountSentLD;     // Amount of tokens ACTUALLY debited from the sender in local decimals.
                                  // @dev In non-default implementations, the amountReceivedLD COULD differ from this value.
        uint256 amountReceivedLD; // Amount of tokens to be received on the remote side.
    }

    #[derive(Debug, PartialEq)]
    struct OFTFeeDetail {
        int256  feeAmountLD;  // Amount of the fee in local decimals.
        string  description;  // Description of the fee.
    }

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct MessagingFee {
        uint256 nativeFee;
        uint256 lzTokenFee;
    }

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct MessagingReceipt {
        bytes32     guid;
        uint64      nonce;
        MessagingFee fee;
    }

    #[derive(Debug, PartialEq)]
    interface IStargate {
        function quoteSend(
            SendParam calldata _sendParam,
            bool               _payInLzToken
        ) external view returns (MessagingFee memory fee);

        /**
         * @notice Provides a quote for sending OFT to another chain.
         * @dev Implements the IOFT interface
         * @param _sendParam The parameters for the send operation
         * @return limit The information on OFT transfer limits
         * @return oftFeeDetails The details of OFT transaction cost or reward
         * @return receipt The OFT receipt information, indicating how many tokens would be sent and received
         */
        function quoteOFT(
            SendParam calldata _sendParam
        ) external view returns (
            OFTLimit           memory limit,
            OFTFeeDetail[]     memory oftFeeDetails,
            OFTReceipt         memory receipt
        );

        /**
         * @notice Executes the send() operation.
         * @param _sendParam The parameters for the send operation.
         * @param _fee The fee information supplied by the caller.
         *      - nativeFee: The native fee.
         *      - lzTokenFee: The lzToken fee.
         * @param _refundAddress The address to receive any excess funds from fees etc. on the src.
         * @return receipt The LayerZero messaging receipt from the send() operation.
         * @return oftReceipt The OFT receipt information.
         *
         * @dev MessagingReceipt: LayerZero msg receipt
         *  - guid: The unique identifier for the sent message.
         *  - nonce: The nonce of the sent message.
         *  - fee: The LayerZero fee incurred for the message.
         */
        function send(
            SendParam      calldata _sendParam,
            MessagingFee   calldata _fee,
            address                 _refundAddress
        ) external payable returns (
            MessagingReceipt memory msgReceipt,
            OFTReceipt       memory oftReceipt
        );
    }
}
