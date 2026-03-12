// Copyright (c). Gem Wallet. All rights reserved.

import Components
import PrimitivesComponents

struct ConfirmHeaderViewModel {
    let headerType: TransactionHeaderType
}

// MARK: - ItemModelProvidable

extension ConfirmHeaderViewModel: ItemModelProvidable {
    var itemModel: ConfirmTransferItemModel {
        .header(
            TransactionHeaderItemModel(
                headerType: headerType,
                showClearHeader: showClearHeader
            )
        )
    }
}

// MARK: - Private

private extension ConfirmHeaderViewModel {
    var showClearHeader: Bool {
        switch headerType {
        case .amount, .nft, .asset, .assetValue: true
        case .swap: false
        }
    }
}
