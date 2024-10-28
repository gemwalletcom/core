use alloy_core::sol;

// https://github.com/Uniswap/permit2/blob/main/src/interfaces/IAllowanceTransfer.sol
sol! {
    /// @title AllowanceTransfer
    /// @notice Handles ERC20 token permissions through signature based allowance setting and ERC20 token transfers by checking allowed amounts
    /// @dev Requires user's token approval on the Permit2 contract
    #[derive(Debug, PartialEq)]
    interface IAllowanceTransfer {
        /// @notice The permit data for a token
        struct PermitDetails {
            // ERC20 token address
            address token;
            // the maximum amount allowed to spend
            uint160 amount;
            // timestamp at which a spender's token allowances become invalid
            uint48 expiration;
            // an incrementing value indexed per owner,token,and spender for each signature
            uint48 nonce;
        }

        /// @notice The permit message signed for a single token allowance
        struct PermitSingle {
            // the permit data for a single token allowance
            PermitDetails details;
            // address permissioned on the allowed tokens
            address spender;
            // deadline on the permit signature
            uint256 sigDeadline;
        }
    }
}
