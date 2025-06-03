use alloy_sol_types::sol;

sol! {
    #[sol(rpc)]
    interface Router {
        function getCurrentRegistrator() external view returns (address);
    }

    #[sol(rpc)]
    interface Registrator {
        function getFullRecordJSON(bytes32 _namehash) external view returns (string);
    }

    #[sol(rpc)]
    interface HyperliquidNames {
        function tokenIdToAddress(uint256 _tokenId) external view returns (address);
    }
}
