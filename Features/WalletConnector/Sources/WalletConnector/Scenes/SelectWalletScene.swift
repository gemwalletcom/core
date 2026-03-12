// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI
import Components
import Style
import PrimitivesComponents
import Primitives

struct SelectWalletScene: View {
    @Environment(\.dismiss) private var dismiss

    @Binding private var model: SelectWalletViewModel

    init(model: Binding<SelectWalletViewModel>) {
        _model = model
    }

    var body: some View {
        SelectableListView(
            model: $model,
            onFinishSelection: onSelect,
            listContent: {
                SimpleListItemView(model: $0)
            }
        )
        .navigationTitle(model.title)
    }
}

// MARK: - Actions

extension SelectWalletScene {
    private func onSelect(wallets: [Wallet]) {
        model.selectedItems = wallets.asSet()
        dismiss()
    }
}
