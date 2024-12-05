use alloy_core::sol;

// https://docs.across.to/reference/selected-contract-functions
// https://github.com/across-protocol/contracts/blob/master/contracts/interfaces/SpokePoolInterface.sol
sol! {
    // Contains structs and functions used by SpokePool contracts to facilitate universal settlement.
    interface V3SpokePoolInterface {
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
    }
}
