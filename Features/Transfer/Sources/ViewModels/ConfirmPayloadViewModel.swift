// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Primitives

struct ConfirmPayloadViewModel {
    private let fields: [SimulationPayloadField]

    init(fields: [SimulationPayloadField]) {
        self.fields = fields
    }
}

// MARK: - ItemModelProvidable

extension ConfirmPayloadViewModel: ItemModelProvidable {
    var itemModel: ConfirmTransferItemModel {
        guard !fields.isEmpty else { return .empty }
        return .payload(fields)
    }
}
