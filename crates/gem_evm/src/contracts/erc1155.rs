use alloy_sol_types::sol;

sol! {
    interface IERC1155 {
        function safeTransferFrom(address from, address to, uint256 id, uint256 amount, bytes data) external;
        function safeBatchTransferFrom(address from, address to, uint256[] ids, uint256[] amounts, bytes data) external;
        function setApprovalForAll(address operator, bool approved) external;
    }
}
