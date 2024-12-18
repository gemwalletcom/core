use alloy_core::sol;

// https://docs.across.to/reference/selected-contract-functions
// https://github.com/across-protocol/contracts/blob/master/contracts/interfaces/SpokePoolInterface.sol
sol! {
    // Contains structs and functions used by SpokePool contracts to facilitate universal settlement.
    interface V3SpokePoolInterface {
        // This struct represents the data to fully specify a **unique** relay submitted on this chain.
        // This data is hashed with the chainId() and saved by the SpokePool to prevent collisions and protect against
        // replay attacks on other chains. If any portion of this data differs, the relay is considered to be
        // completely distinct.
        struct V3RelayData {
            // The address that made the deposit on the origin chain.
            address depositor;
            // The recipient address on the destination chain.
            address recipient;
            // This is the exclusive relayer who can fill the deposit before the exclusivity deadline.
            address exclusiveRelayer;
            // Token that is deposited on origin chain by depositor.
            address inputToken;
            // Token that is received on destination chain by recipient.
            address outputToken;
            // The amount of input token deposited by depositor.
            uint256 inputAmount;
            // The amount of output token to be received by recipient.
            uint256 outputAmount;
            // Origin chain id.
            uint256 originChainId;
            // The id uniquely identifying this deposit on the origin chain.
            uint32 depositId;
            // The timestamp on the destination chain after which this deposit can no longer be filled.
            uint32 fillDeadline;
            // The timestamp on the destination chain after which any relayer can fill the deposit.
            uint32 exclusivityDeadline;
            // Data that is forwarded to the recipient.
            bytes message;
        }

        function depositV3(
            address depositor,
            address recipient,
            address inputToken,
            address outputToken,
            uint256 inputAmount,
            uint256 outputAmount,
            uint256 destinationChainId,
            address exclusiveRelayer,
            uint32 quoteTimestamp,
            uint32 fillDeadline,
            uint32 exclusivityDeadline,
            bytes calldata message
        ) external payable;

        function fillV3Relay(V3RelayData calldata relayData, uint256 repaymentChainId) external;
    }
}
