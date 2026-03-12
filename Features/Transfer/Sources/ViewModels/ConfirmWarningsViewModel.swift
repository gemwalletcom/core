// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Primitives
import PrimitivesComponents

struct ConfirmWarningsViewModel {
    private let warnings: [SimulationWarning]

    init(warnings: [SimulationWarning]) {
        self.warnings = warnings
    }
}

// MARK: - ItemModelProvidable

extension ConfirmWarningsViewModel: ItemModelProvidable {
    var itemModel: ConfirmTransferItemModel {
        guard !warnings.isEmpty else { return .empty }
        return .warnings(warnings)
    }
}
