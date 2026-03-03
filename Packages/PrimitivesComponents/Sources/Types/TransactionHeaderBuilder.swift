// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import BigInt
import Primitives

public struct TransactionHeaderTypeBuilder {
    public static func build(
        infoModel: TransactionInfoViewModel,
        transaction: Transaction,
        metadata: TransactionExtendedMetadata?
    ) -> TransactionHeaderType {
        let inputType: TransactionHeaderInputType = {
            switch transaction.type {
            case .transfer,
                    .stakeDelegate,
                    .stakeUndelegate,
                    .stakeRedelegate,
                    .stakeRewards,
                    .stakeWithdraw,
                    .smartContractCall,
                    .stakeFreeze,
                    .stakeUnfreeze:
                return .amount(showFiat: true)
            case .swap:
                guard let metadata, let input = SwapMetadataViewModel(metadata: metadata).headerInput else {
                    fatalError("swapMetadata is missed")
                }
                return .swap(input)
            case .assetActivation:
                return .symbol
            case .tokenApproval:
                if infoModel.isZero {
                    return .amount(showFiat: false)
                } else {
                    return .symbol
                }
            case .transferNFT:
                guard let metadata = transaction.metadata?.decode(TransactionNFTTransferMetadata.self) else {
                    return .amount(showFiat: false)
                }
                return .nft(name: metadata.name, id: metadata.assetId)
            case .perpetualOpenPosition, .perpetualClosePosition, .perpetualModifyPosition:
                return .symbol
            case .earnDeposit, .earnWithdraw:
                return .amount(showFiat: true)
            }
        }()
        return infoModel.headerType(input: inputType)
    }

    public static func build(
        infoModel: TransactionInfoViewModel,
        dataType: TransferDataType,
        metadata: TransferDataMetadata?
    ) -> TransactionHeaderType {
        let inputType: TransactionHeaderInputType = {
            switch dataType {
            case .transfer,
                    .deposit,
                    .withdrawal,
                    .generic,
                    .stake,
                    .tokenApprove:
                return .amount(
                    showFiat: true
                )
            case .transferNft(let asset):
                return .nft(name: asset.name, id: asset.id)
            case .account(_, let type):
                switch type {
                case .activate:
                    return .amount(
                        showFiat: false
                    )
                }
            case .swap(let fromAsset, let toAsset, let data):
                let assetPrices = (metadata?.assetPrices ?? [:]).map { (assetId, price) in
                    price.mapToAssetPrice(assetId: assetId)
                }

                let model = SwapMetadataViewModel(
                    metadata: TransactionExtendedMetadata(
                        assets: [fromAsset, toAsset],
                        assetPrices: assetPrices,
                        metadata: .encode(TransactionSwapMetadata(
                            fromAsset: fromAsset.id,
                            fromValue: data.quote.fromValue,
                            toAsset: toAsset.id,
                            toValue: data.quote.toValue,
                            provider: data.quote.providerData.provider.rawValue
                        ))
                    )
                )

                guard let input = model.headerInput else {
                    fatalError("fromAsset & toAsset missed")
                }
                return .swap(input)
            case .perpetual:
                return .symbol
            case .earn:
                return .amount(showFiat: true)
            }
        }()
        return infoModel.headerType(input: inputType)
    }
}
