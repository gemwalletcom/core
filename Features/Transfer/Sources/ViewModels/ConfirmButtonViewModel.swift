// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Localization
import Primitives
import Style
import SwiftUI

struct ConfirmButtonViewModel: StateButtonViewable {
    private let onAction: @MainActor @Sendable () -> Void
    private let state: StateViewType<TransactionInputViewModel>
    private let isDisabled: Bool

    let icon: Image?

    init(
        state: StateViewType<TransactionInputViewModel>,
        icon: Image?,
        isDisabled: Bool = false,
        onAction: @MainActor @Sendable @escaping () -> Void
    ) {
        self.state = state
        self.icon = icon
        self.isDisabled = isDisabled
        self.onAction = onAction
    }

    var title: String {
        state.isError ? Localized.Common.tryAgain : Localized.Transfer.confirm
    }

    var type: ButtonType {
        let isDisabled = isDisabled || (state.value?.transferAmount?.isFailure ?? false)
        return .primary(state, isDisabled: isDisabled)
    }

    func action() {
        onAction()
    }
}
