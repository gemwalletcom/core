use alloy_sol_types::sol;

pub mod v3;
pub mod v4;

// https://github.com/Uniswap/universal-router/blob/main/contracts/interfaces/IUniversalRouter.sol
// https://github.com/Uniswap/universal-router/blob/main/contracts/base/Dispatcher.sol
sol! {
    /// @notice Executes encoded commands along with provided inputs. Reverts if deadline has expired.
    /// @param commands A set of concatenated commands, each 1 byte in length
    /// @param inputs An array of byte strings containing abi encoded inputs for each command
    /// @param deadline The deadline by which the transaction must be executed
    #[derive(Debug, PartialEq)]
    interface IUniversalRouter {
        function execute(bytes calldata commands, bytes[] calldata inputs, uint256 deadline) external payable;
    }
}
