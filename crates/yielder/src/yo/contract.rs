use alloy_sol_types::sol;

sol! {
    interface IYoGateway {
        function quoteConvertToShares(address yoVault, uint256 assets) external view returns (uint256 shares);
        function deposit(address yoVault, uint256 assets, uint256 minSharesOut, address receiver, uint32 partnerId) external returns (uint256 sharesOut);
        function redeem(address yoVault, uint256 shares, uint256 minAssetsOut, address receiver, uint32 partnerId) external returns (uint256 assetsOrRequestId);
    }
}
