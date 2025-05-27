use alloy_sol_types::sol;

sol! {
    /// Interface for the L2Resolver
    interface L2Resolver {
        /// Returns the address associated with a node.
        /// @param node The ENS node to query.
        /// @return The associated address.
        function addr(bytes32 node) external view returns (address);

        /// Returns the text record associated with an ENS node and key.
        /// @param node The ENS node to query.
        /// @param key The text record key.
        /// @return The text record.
        function text(bytes32 node, string key) external view returns (string);
    }
}
