// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI
import Components
import Style

public struct ChartListView<Model: ChartListViewable, Content: View>: View {
    @Bindable var model: Model
    @ViewBuilder let content: () -> Content

    public init(model: Model, @ViewBuilder content: @escaping () -> Content) {
        self.model = model
        self.content = content
    }

    public var body: some View {
        List {
            Section { } header: {
                ChartStateView(
                    state: model.chartState,
                    selectedPeriod: $model.selectedPeriod,
                    periods: model.periods
                )
            }
            .cleanListRow()
            content()
        }
        .listSectionSpacing(.compact)
        .task(id: model.selectedPeriod) {
            await model.fetch()
        }
        .refreshableTimer(every: .minutes(1)) { @MainActor in
            await model.fetch()
        }
    }
}
