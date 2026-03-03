// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import BigInt

extension Transaction {

    public static func id(chain: Chain, hash: String) -> String {
        return String(format: "%@_%@", chain.rawValue, hash)
    }
    
    public var chain: Chain {
        assetId.chain
    }

    public var valueBigInt: BigInt {
        return BigInt(value) ?? .zero
    }

    public var feeBigInt: BigInt {
        return BigInt(fee) ?? .zero
    }

    public var assetIds: [AssetId] {
        switch type {
        case .transfer,
            .tokenApproval,
            .stakeDelegate,
            .stakeUndelegate,
            .stakeRedelegate,
            .stakeRewards,
            .stakeWithdraw,
            .assetActivation,
            .transferNFT,
            .smartContractCall,
            .perpetualOpenPosition,
            .perpetualClosePosition,
            .perpetualModifyPosition,
            .stakeFreeze,
            .stakeUnfreeze,
            .earnDeposit,
            .earnWithdraw:
            return [assetId]
        case .swap:
            guard let swapMetadata = metadata?.decode(TransactionSwapMetadata.self) else {
                return []
            }
            return [swapMetadata.fromAsset, swapMetadata.toAsset]
        }
    }

    public var swapProvider: String? {
        metadata?.decode(TransactionSwapMetadata.self)?.provider
    }
}

extension Transaction: Identifiable { }
