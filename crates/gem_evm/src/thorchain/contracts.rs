use alloy_sol_types::sol;

sol! {
    interface RouterInterface {
        function depositWithExpiry(address inbound_address, address token_address, uint amount, string memo, uint expiry) external;
    }
}
