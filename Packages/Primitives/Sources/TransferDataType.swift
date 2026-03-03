// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public enum TransferDataType: Hashable, Equatable, Sendable {
    case transfer(Asset)
    case deposit(Asset)
    case withdrawal(Asset)
    case transferNft(NFTAsset)
    case swap(Asset, Asset, SwapData)
    case tokenApprove(Asset, ApprovalData)
    case stake(Asset, StakeType)
    case account(Asset, AccountDataType)
    case perpetual(Asset, PerpetualType)
    case earn(Asset, EarnType, ContractCallData)
    case generic(asset: Asset, metadata: WalletConnectionSessionAppMetadata, extra: TransferDataExtra)

    public var transactionType: TransactionType {
        switch self {
        case .transfer: .transfer
        case .deposit: .transfer
        case .withdrawal: .transfer
        case .generic: .smartContractCall
        case .transferNft: .transferNFT
        case .tokenApprove: .tokenApproval
        case .swap: .swap
        case .stake(_, let type):
            switch type {
            case .stake: .stakeDelegate
            case .unstake: .stakeUndelegate
            case .redelegate: .stakeRedelegate
            case .rewards: .stakeRewards
            case .withdraw: .stakeWithdraw
            case .freeze(let data):
                switch data.freezeType {
                case .freeze: .stakeFreeze
                case .unfreeze: .stakeUnfreeze
                }
            }
        case .account: .assetActivation
        case .earn(_, let type, _):
            switch type {
            case .deposit: .earnDeposit
            case .withdraw: .earnWithdraw
            }
        case .perpetual(_, let type):
            switch type {
            case .open, .increase: .perpetualOpenPosition
            case .close, .reduce: .perpetualClosePosition
            case .modify: .perpetualModifyPosition
            }
        }
    }

    public var chain: Chain {
        switch self {
        case .transfer(let asset),
             .deposit(let asset),
             .withdrawal(let asset),
             .swap(let asset, _, _),
             .stake(let asset, _),
             .account(let asset, _),
             .perpetual(let asset, _),
             .earn(let asset, _, _),
             .tokenApprove(let asset, _),
             .generic(let asset, _, _): asset.chain
        case .transferNft(let asset): asset.chain
        }
    }

    public var metadata: AnyCodableValue? {
        switch self {
        case .swap(let fromAsset, let toAsset, let data):
            return .encode(TransactionSwapMetadata(
                fromAsset: fromAsset.id,
                fromValue: data.quote.fromValue,
                toAsset: toAsset.id,
                toValue: data.quote.toValue,
                provider: data.quote.providerData.provider.rawValue
            ))
        case .transferNft(let asset):
            return .encode(TransactionNFTTransferMetadata(assetId: asset.id, name: asset.name))
        case .perpetual(_, let type):
            guard let direction = type.data?.direction else { return nil }
            return .encode(TransactionPerpetualMetadata(pnl: 0, price: 0, direction: direction, provider: nil))
        case .stake(_, let type):
            switch type {
            case .freeze(let data):
                return .encode(TransactionResourceTypeMetadata(resourceType: data.resource))
            case .stake, .unstake, .redelegate, .rewards, .withdraw:
                return nil
            }
        case .generic(_, _, let extra):
            return .encode(TransactionWalletConnectMetadata(outputAction: extra.outputAction))
        case .transfer,
            .deposit,
            .withdrawal,
            .tokenApprove,
            .account,
            .earn:
            return nil
        }
    }

    public var assetIds: [AssetId] {
        switch self {
        case .transfer(let asset),
             .deposit(let asset),
             .withdrawal(let asset),
             .tokenApprove(let asset, _),
             .stake(let asset, _),
             .generic(let asset, _, _),
             .account(let asset, _),
             .perpetual(let asset, _),
             .earn(let asset, _, _): [asset.id]
        case .swap(let from, let to, _): [from.id, to.id]
        case .transferNft: []
        }
    }

    public var outputType: TransferDataOutputType {
        return switch self {
        case .generic(_, _, let extra): extra.outputType
        default: .encodedTransaction
        }
    }

    public var outputAction: TransferDataOutputAction {
        return switch self {
        case .generic(_, _, let extra): extra.outputAction
        default: .send
        }
    }

    public func swap() throws -> (Asset, Asset, data: SwapData) {
        guard case .swap(let fromAsset, let toAsset, let data) = self else {
            throw AnyError("SwapQuoteData missed")
        }
        return (fromAsset, toAsset, data)
    }

    public func earn() throws -> (Asset, EarnType, data: ContractCallData) {
        guard case .earn(let asset, let earnType, let data) = self else {
            throw AnyError("EarnData missed")
        }
        return (asset, earnType, data)
    }

    public var shouldIgnoreValueCheck: Bool {
        switch self {
        case .transferNft, .stake, .account, .tokenApprove, .perpetual, .earn: true
        case .transfer, .deposit, .withdrawal, .swap, .generic: false
        }
    }

    public func withGasLimit(_ gasLimit: String) -> TransferDataType {
        guard case .swap(let from, let to, let swapData) = self else { return self }
        return .swap(from, to, swapData.withGasLimit(gasLimit))
    }

    public var recentActivityData: RecentActivityData? {
        switch self {
        case .transfer(let asset): RecentActivityData(type: .transfer, assetId: asset.id, toAssetId: nil)
        case .swap(let from, let to, _): RecentActivityData(type: .swap, assetId: from.id, toAssetId: to.id)
        default: nil
        }
    }
}
