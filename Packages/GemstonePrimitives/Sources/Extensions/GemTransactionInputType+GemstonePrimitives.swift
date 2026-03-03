// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import Gemstone
import Primitives

public extension GemTransactionInputType {
    func getAsset() -> GemAsset {
        switch self {
        case .transfer(let asset): asset
        case .deposit(let asset): asset
        case .transferNft(let asset, _): asset
        case .swap(let fromAsset, _, _): fromAsset
        case .stake(let asset, _): asset
        case .tokenApprove(let asset, _): asset
        case .generic(let asset, _, _): asset
        case .account(let asset, _): asset
        case .perpetual(asset: let asset, perpetualType: _): asset
        case .earn(asset: let asset, earnType: _, data: _): asset
        }
    }
}

public extension GemTransactionInputType {
    func map() throws -> TransferDataType {
        switch self {
        case .transfer(let asset):
            return try TransferDataType.transfer(asset.map())
        case .deposit(let asset):
            return try TransferDataType.deposit(asset.map())
        case .swap(let fromAsset, let toAsset, let gemSwapData):
            return try TransferDataType.swap(fromAsset.map(), toAsset.map(), gemSwapData.map())
        case .transferNft(_, let nftAsset):
            return try TransferDataType.transferNft(nftAsset.map())
        case .stake(let asset, let type):
            return try TransferDataType.stake(asset.map(), type.map())
        case .tokenApprove(let asset, let approvalData):
            return try TransferDataType.tokenApprove(asset.map(), approvalData.map())
        case .generic(let asset, let metadata, let extra):
            return try TransferDataType.generic(asset: asset.map(), metadata: metadata.map(), extra: extra.map())
        case .account(let asset, let accountType):
            return try TransferDataType.account(asset.map(), accountType.map())
        case .perpetual(asset: let asset, perpetualType: let perpetualType):
            return try TransferDataType.perpetual(asset.map(), perpetualType.map())
        case .earn(let asset, let earnType, let data):
            return try TransferDataType.earn(asset.map(), earnType.map(), data.map())
        }
    }
}

public extension TransferDataType {
    func map() throws -> GemTransactionInputType {
        switch self {
        case .transfer(let asset):
            return .transfer(asset: asset.map())
        case .deposit(let asset):
            return .deposit(asset: asset.map())
        case .swap(let fromAsset, let toAsset, let swapData):
            return try .swap(fromAsset: fromAsset.map(), toAsset: toAsset.map(), swapData: swapData.map())
        case .transferNft(let nftAsset):
            return .transferNft(asset: Asset(nftAsset.chain).map(), nftAsset: nftAsset.map())
        case .stake(let asset, let stakeType):
            return .stake(asset: asset.map(), stakeType: stakeType.map())
        case .tokenApprove(let asset, let approvalData):
            return .tokenApprove(asset: asset.map(), approvalData: approvalData.map())
        case .generic(let asset, let metadata, let extra):
            return .generic(asset: asset.map(), metadata: metadata.map(), extra: extra.map())
        case .withdrawal(let asset):
            if asset.chain == .hyperCore {
                return .transfer(asset: asset.map())
            }
            throw AnyError("Unsupported transaction type: \(self)")
        case .account(let asset, let accountData):
            return .account(asset: asset.map(), accountType: accountData.map())
        case .perpetual(let asset, let perpetualType):
            return .perpetual(asset: asset.map(), perpetualType: perpetualType.map())
        case .earn(let asset, let earnType, let data):
            return .earn(asset: asset.map(), earnType: earnType.map(), data: data.map())
        }
    }
}
