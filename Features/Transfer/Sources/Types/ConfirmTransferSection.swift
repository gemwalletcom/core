// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import SwiftUI
import Components
import PrimitivesComponents
import Swap
import Primitives

enum ConfirmTransferSectionType: String, Identifiable, Equatable {
    case header
    case warnings
    case details
    case payload
    case fee
    case error

    public var id: String { rawValue }
}

public enum ConfirmTransferItem: Identifiable, Equatable, Sendable {
    case header
    case warnings
    case app
    case sender
    case network
    case recipient
    case memo
    case details
    case payload
    case networkFee
    case error

    public var id: Self { self }
}

public enum ConfirmTransferItemModel {
    case app(ListItemModel)
    case sender(ListItemModel)
    case header(TransactionHeaderItemModel)
    case recipient(AddressListItemViewModel)
    case network(ListItemModel)
    case memo(ListItemModel)
    case swapDetails(SwapDetailsViewModel)
    case networkFee(ListItemModel, selectable: Bool)
    case perpetualDetails(PerpetualDetailsViewModel)
    case perpetualModifyPosition(PerpetualModifyViewModel)
    case warnings([SimulationWarning])
    case payload([SimulationPayloadField])
    case error(title: String, error: Error, onInfoAction: VoidAction)
    case empty
}

extension ConfirmTransferItemModel: ItemModelProvidable {
    public var itemModel: ConfirmTransferItemModel { self }
}

extension ListSection where T == ConfirmTransferItem {
    init(type: ConfirmTransferSectionType, _ items: [ConfirmTransferItem]) {
        self.init(type: type, values: items)
    }
}
