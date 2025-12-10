use alloy_sol_types::sol;

sol! {
    interface Router {
        function getCurrentRegistrator() external view returns (address);
    }

    interface Registrator {
        function getFullRecordJSON(bytes32 _namehash) external view returns (string);
    }
}
