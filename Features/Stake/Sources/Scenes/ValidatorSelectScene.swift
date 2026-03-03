// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI
import Components
import Style
import Primitives
import PrimitivesComponents

public struct ValidatorSelectScene: View {
    @Environment(\.dismiss) private var dismiss
    
    @State private var model: ValidatorSelectSceneViewModel

    public init(model: ValidatorSelectSceneViewModel) {
        _model = State(wrappedValue: model)
    }

    public var body: some View {
        List {
            ForEach(model.list) { section in
                Section(section.section) {
                    ForEach(section.values) { value in
                        ValidatorSelectionView(value: value, selection: model.currentValidator?.id) {
                            model.selectValidator?($0)
                            dismiss()
                        }
                        .contextMenu(model.contextMenu(for: value.value))
                    }
                }
            }
        }
        .navigationTitle(model.title)
        .safariSheet(url: $model.isPresentingUrl)
    }
}
