use alloy_core::sol;
use serde::{Deserialize, Serialize};

sol! {
    #[derive(Debug)]
    struct Call {
        address target;
        bytes callData;
        uint256 value;
    }

    #[derive(Debug)]
    struct Instructions {
        address token;
        //  Calls that will be attempted.
        Call[] calls;
        // Where the tokens go if any part of the call fails.
        // Leftover tokens are sent here as well if the action succeeds.
        address fallbackRecipient;
    }

    /// Parameters for the OFT send() operation
    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct SendParam {
        uint32  dstEid;
        bytes32 to;
        uint256 amountLD;
        uint256 minAmountLD;
        bytes   extraOptions;
        bytes   composeMsg;
        bytes   oftCmd;
    }

    /// OFT limit information
    ///
    /// These amounts can change dynamically and are determined by the specific OFT implementation
    #[derive(Debug, PartialEq)]
    struct OFTLimit {
        uint256 minAmountLD;
        uint256 maxAmountLD;
    }

    /// OFT receipt information containing details about sent and received amounts
    #[derive(Debug, PartialEq)]
    struct OFTReceipt {
        uint256 amountSentLD;
        uint256 amountReceivedLD;
    }

    /// Detailed information about OFT fees
    #[derive(Debug, PartialEq)]
    struct OFTFeeDetail {
        int256  feeAmountLD;
        string  description;
    }

    /// Structure containing messaging fee information
    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct MessagingFee {
        uint256 nativeFee;
        uint256 lzTokenFee;
    }

    /// Receipt for messaging operations
    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct MessagingReceipt {
        bytes32     guid;
        uint64      nonce;
        MessagingFee fee;
    }

    /// Interface for Stargate cross-chain operations
    #[derive(Debug, PartialEq)]
    interface IStargate {
        /// Provides a quote for messaging fees
        ///
        /// # Arguments
        ///
        /// * `_sendParam` - Parameters for the send operation
        /// * `_payInLzToken` - Flag indicating whether to pay in LayerZero tokens
        ///
        /// # Returns
        ///
        /// * `fee` - Messaging fee information
        function quoteSend(
            SendParam calldata _sendParam,
            bool               _payInLzToken
        ) external view returns (MessagingFee memory fee);

        /// Provides a quote for sending OFT to another chain
        ///
        /// # Arguments
        ///
        /// * `_sendParam` - Parameters for the send operation
        ///
        /// # Returns
        ///
        /// * `limit` - Information on OFT transfer limits
        /// * `oftFeeDetails` - Details of OFT transaction cost or reward
        /// * `receipt` - OFT receipt information indicating token amounts
        function quoteOFT(
            SendParam calldata _sendParam
        ) external view returns (
            OFTLimit           memory limit,
            OFTFeeDetail[]     memory oftFeeDetails,
            OFTReceipt         memory receipt
        );

        /// Executes the send operation
        ///
        /// # Arguments
        ///
        /// * `_sendParam` - Parameters for the send operation
        /// * `_fee` - Fee information containing native and LayerZero token fees
        /// * `_refundAddress` - Address to receive any excess funds from fees on the source chain
        ///
        /// # Returns
        ///
        /// * `msgReceipt` - LayerZero messaging receipt containing:
        ///   - guid: Unique identifier for the message
        ///   - nonce: Message nonce
        ///   - fee: LayerZero fee details
        /// * `oftReceipt` - OFT receipt with sent and received amount information
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
