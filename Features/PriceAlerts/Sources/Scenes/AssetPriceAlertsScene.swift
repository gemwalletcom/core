// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI
import Store
import Primitives
import Style
import Components
import Localization
import PrimitivesComponents

public struct AssetPriceAlertsScene: View {
    @State private var model: AssetPriceAlertsViewModel

    public init(model: AssetPriceAlertsViewModel) {
        _model = State(initialValue: model)
    }
    
    public var body: some View {
        List {
            Section {
                autoAlertToggleView
            } footer: {
                Text(Localized.PriceAlerts.autoFooter)
            }

            if model.alertsModel.isNotEmpty {
                Section {
                    ForEach(model.alertsModel, id: \.data.priceAlert.id) { alertModel in
                        alertView(model: alertModel)
                    }
                } header: {
                    Text(Localized.Stake.active)
                }
            }
        }
        .bindQuery(model.query)
        .bindQuery(model.priceQuery)
        .listSectionSpacing(.compact)
        .refreshable { await model.fetch() }
        .task { await model.fetch() }
        .navigationTitle(model.title)
        .navigationBarTitleDisplayMode(.inline)
        .toolbar {
            ToolbarItem(placement: .navigationBarTrailing) {
                Button(action: model.onSelectSetPriceAlert) {
                    Image(systemName: SystemImage.plus)
                }
            }
        }
        .sheet(isPresented: $model.isPresentingSetPriceAlert) {
            SetPriceAlertNavigationStack(
                model: SetPriceAlertViewModel(
                    walletId: model.walletId,
                    asset: model.asset,
                    priceAlertService: model.priceAlertService
                ) { model.onSetPriceAlertComplete(message: $0) }
            )
        }
        .toast(message: $model.isPresentingToastMessage)
    }
    
    private var autoAlertToggleView: some View {
        Toggle(isOn: model.isAutoAlertEnabledBinding) {
            ListAssetItemView(model: model.autoAlertItemModel)
        }
        .toggleStyle(AppToggleStyle())
    }

    private func alertView(model: PriceAlertItemViewModel) -> some View {
        ListAssetItemView(model: model)
            .swipeActions(edge: .trailing) {
                Button(Localized.Common.delete, role: .destructive) {
                    onDelete(alert: model.data.priceAlert)
                }
                .tint(Colors.red)
            }
    }
}

// MARK: - Actions

extension AssetPriceAlertsScene {
    private func onDelete(alert: PriceAlert) {
        Task {
            await model.deletePriceAlert(priceAlert: alert)
        }
    }
}
