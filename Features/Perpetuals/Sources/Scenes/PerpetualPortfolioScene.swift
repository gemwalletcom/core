// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI
import Primitives
import Components
import Style
import PrimitivesComponents

struct PerpetualPortfolioScene: View {
    @State private var model: PerpetualPortfolioSceneViewModel

    init(model: PerpetualPortfolioSceneViewModel) {
        _model = State(initialValue: model)
    }

    var body: some View {
        NavigationStack {
            List {
                Section { } header: {
                    ChartStateView(
                        state: model.chartState,
                        selectedPeriod: $model.selectedPeriod,
                        periods: model.periods
                    )
                }
                .cleanListRow()

                Section(header: Text(model.infoSectionTitle)) {
                    ListItemView(field: model.unrealizedPnlField)
                    ListItemView(field: model.accountLeverageField)
                    ListItemView(field: model.marginUsageField)
                    ListItemView(field: model.allTimePnlField)
                    ListItemView(field: model.volumeField)
                }
            }
            .navigationTitle(model.navigationTitle)
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .topBarTrailing) {
                    Menu {
                        Picker("", selection: $model.selectedChartType) {
                            ForEach(PerpetualPortfolioChartType.allCases) { type in
                                Text(model.chartTypeTitle(type)).tag(type)
                            }
                        }
                    } label: {
                        Text(model.chartTypeTitle(model.selectedChartType))
                            .fontWeight(.semibold)
                    }
                }
            }
            .toolbarDismissItem(type: .close, placement: .cancellationAction)
            .task {
                await model.fetch()
            }
            .refreshableTimer(every: .minutes(1)) {
                await model.fetch()
            }
            .listSectionSpacing(.compact)
        }
    }
}
