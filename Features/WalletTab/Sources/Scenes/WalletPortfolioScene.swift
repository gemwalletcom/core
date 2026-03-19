// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI
import Components
import Localization
import PrimitivesComponents
import Store

public struct WalletPortfolioScene: View {
    @State private var model: WalletPortfolioSceneViewModel

    public init(model: WalletPortfolioSceneViewModel) {
        _model = State(initialValue: model)
    }

    public var body: some View {
        NavigationStack {
            ChartListView(model: model) {
                if model.allTimeValues.isNotEmpty {
                    Section {
                        ForEach(model.allTimeValues, id: \.title) {
                            ListItemView(model: $0)
                        }
                    }
                }
            }
            .navigationTitle(model.navigationTitle)
            .navigationBarTitleDisplayMode(.inline)
            .toolbarDismissItem(type: .close, placement: .cancellationAction)
            .bindQuery(model.assetsQuery)
        }
    }
}
