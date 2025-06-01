use alloy_sol_types::sol;

sol! {
    #[sol(rpc)]
    interface Router {
        function getCurrentRegistrator() external view returns (address);
    }

    #[sol(rpc)]
    interface Registrator {
        function ownerOf(bytes32 _namehash) external view returns (address);
    }
}
