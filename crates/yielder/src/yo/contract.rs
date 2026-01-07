use alloy_sol_types::sol;

sol! {
    interface IYoVaultToken {
        function convertToAssets(uint256 shares) external view returns (uint256 assets);
    }

    interface IYoGateway {
        function quoteConvertToShares(address yoVault, uint256 assets) external view returns (uint256 shares);

        function quoteConvertToAssets(address yoVault, uint256 shares) external view returns (uint256 assets);

        function quotePreviewDeposit(address yoVault, uint256 assets) external view returns (uint256 shares);

        function quotePreviewRedeem(address yoVault, uint256 shares) external view returns (uint256 assets);

        function getAssetAllowance(address yoVault, address owner) external view returns (uint256 allowance);

        function getShareAllowance(address yoVault, address owner) external view returns (uint256 allowance);

        function deposit(
            address yoVault,
            uint256 assets,
            uint256 minSharesOut,
            address receiver,
            uint32 partnerId
        )
            external
            returns (uint256 sharesOut);

        function redeem(
            address yoVault,
            uint256 shares,
            uint256 minAssetsOut,
            address receiver,
            uint32 partnerId
        )
            external
            returns (uint256 assetsOrRequestId);
    }
}
