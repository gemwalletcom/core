use alloy_sol_types::sol;

// https://github.com/across-protocol/contracts/blob/master/contracts/handlers/MulticallHandler.sol
sol! {
    struct Call {
        address target;
        bytes callData;
        uint256 value;
    }

    struct Instructions {
        //  Calls that will be attempted.
        Call[] calls;
        // Where the tokens go if any part of the call fails.
        // Leftover tokens are sent here as well if the action succeeds.
        address fallbackRecipient;
    }
}
