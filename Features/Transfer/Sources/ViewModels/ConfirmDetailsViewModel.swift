// Copyright (c). Gem Wallet. All rights reserved.

import Primitives
import PrimitivesComponents
import Components
import Swap

struct ConfirmDetailsViewModel {
    private let type: TransferDataType
    private let metadata: TransferDataMetadata?

    init(type: TransferDataType, metadata: TransferDataMetadata?) {
        self.type = type
        self.metadata = metadata
    }
}

// MARK: - ItemModelProvidable

extension ConfirmDetailsViewModel: ItemModelProvidable {
    var itemModel: ConfirmTransferItemModel {
        switch type {
        case let .swap(fromAsset, toAsset, swapData):
                .swapDetails(
                    SwapDetailsViewModel(
                        fromAssetPrice: AssetPriceValue(asset: fromAsset, price: metadata?.assetPrice),
                        toAssetPrice: AssetPriceValue(asset: toAsset, price: metadata?.assetPrices[toAsset.id]),
                        selectedQuote: swapData.quote
                    )
                )
        case let .perpetual(_, perpetualType):
            switch perpetualType {
            case .open, .close, .increase, .reduce:
                    .perpetualDetails(PerpetualDetailsViewModel(type: PerpetualDetailsType(perpetualType)))
            case .modify(let data):
                .perpetualModifyPosition(PerpetualModifyViewModel(data: data))
            }
        case .transfer,
            .deposit,
            .withdrawal,
            .transferNft,
            .tokenApprove,
            .stake,
            .account,
            .generic,
            .earn:
            .empty
        }
    }
}
