use alloy_sol_types::sol;
use primitives::eip712::EIP712Type;
use serde::{Deserialize, Serialize};

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

        /// @notice A mapping from owner address to token address to spender address to PackedAllowance struct, which contains details and conditions of the approval.
        /// @notice The mapping is indexed in the above order see: allowance[ownerAddress][tokenAddress][spenderAddress]
        /// @dev The packed slot holds the allowed amount, expiration at which the allowed amount is no longer valid, and current nonce thats updated on any signature based approvals.
        function allowance(address, address, address) external view returns (uint160, uint48, uint48);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permit2Types {
    #[serde(rename = "EIP712Domain")]
    pub eip712_domain: Vec<EIP712Type>,
    #[serde(rename = "PermitSingle")]
    pub single_types: Vec<EIP712Type>,
    #[serde(rename = "PermitDetails")]
    pub details_types: Vec<EIP712Type>,
}

impl Default for Permit2Types {
    fn default() -> Self {
        Self {
            eip712_domain: vec![
                EIP712Type {
                    name: "name".into(),
                    r#type: "string".into(),
                },
                EIP712Type {
                    name: "chainId".into(),
                    r#type: "uint256".into(),
                },
                EIP712Type {
                    name: "verifyingContract".into(),
                    r#type: "address".into(),
                },
            ],
            single_types: vec![
                EIP712Type {
                    name: "details".into(),
                    r#type: "PermitDetails".into(),
                },
                EIP712Type {
                    name: "spender".into(),
                    r#type: "address".into(),
                },
                EIP712Type {
                    name: "sigDeadline".into(),
                    r#type: "uint256".into(),
                },
            ],
            details_types: vec![
                EIP712Type {
                    name: "token".into(),
                    r#type: "address".into(),
                },
                EIP712Type {
                    name: "amount".into(),
                    r#type: "uint160".into(),
                },
                EIP712Type {
                    name: "expiration".into(),
                    r#type: "uint48".into(),
                },
                EIP712Type {
                    name: "nonce".into(),
                    r#type: "uint48".into(),
                },
            ],
        }
    }
}
