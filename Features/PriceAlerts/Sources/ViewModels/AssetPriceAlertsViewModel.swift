// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI
import Store
import Primitives
import Localization
import PriceAlertService
import PrimitivesComponents
import Components
import Style
import Preferences

@Observable
@MainActor
public final class AssetPriceAlertsViewModel: Sendable {
    let priceAlertService: PriceAlertService
    let walletId: WalletId
    let asset: Asset

    public let query: ObservableQuery<PriceAlertsRequest>
    public let priceQuery: ObservableQuery<PriceRequest>
    var priceAlerts: [PriceAlertData] { query.value }

    var isPresentingSetPriceAlert: Bool = false
    var isPresentingToastMessage: ToastMessage?

    public init(
        priceAlertService: PriceAlertService,
        walletId: WalletId,
        asset: Asset
    ) {
        self.priceAlertService = priceAlertService
        self.walletId = walletId
        self.asset = asset
        self.query = ObservableQuery(PriceAlertsRequest(assetId: asset.id), initialValue: [])
        self.priceQuery = ObservableQuery(PriceRequest(assetId: asset.id), initialValue: nil)
    }
    
    var title: String { Localized.Settings.PriceAlerts.title }

    var autoAlertItemModel: PriceAlertItemViewModel {
        PriceAlertItemViewModel(data: PriceAlertData(
            asset: asset,
            price: priceQuery.value?.price,
            priceAlert: .default(for: asset.id, currency: Preferences.standard.currency)
        ))
    }

    var isAutoAlertEnabledBinding: Binding<Bool> {
        Binding(
            get: { self.priceAlerts.contains(where: { $0.priceAlert.type == .auto }) },
            set: { newValue in
                Task { await self.toggleAutoAlert(enabled: newValue) }
            }
        )
    }

    var alertsModel: [PriceAlertItemViewModel] {
        priceAlerts
            .filter { $0.priceAlert.shouldDisplay && $0.priceAlert.type != .auto }
            .sorted(using: [
                KeyPathComparator(\.priceAlert.price, order: .reverse),
                KeyPathComparator(\.priceAlert.priceDirection, order: .reverse),
                KeyPathComparator(\.priceAlert.pricePercentChange, order: .reverse)
            ])
            .map { PriceAlertItemViewModel(data: $0) }
    }
}

// MARK: - Business Logic

extension AssetPriceAlertsViewModel {
    func fetch() async {
        do {
            try await priceAlertService.update(assetId: asset.id.identifier)
        } catch {
            debugLog("fetch error: \(error)")
        }
    }
    
    func toggleAutoAlert(enabled: Bool) async {
        let currency = Preferences.standard.currency
        do {
            if enabled {
                try await priceAlertService.add(priceAlert: .default(for: asset.id, currency: currency))
                try await priceAlertService.requestPermissions()
                try await priceAlertService.enablePriceAlerts()
            } else {
                try await priceAlertService.delete(priceAlerts: [.default(for: asset.id, currency: currency)])
            }
        } catch {
            debugLog("toggleAutoAlert error: \(error)")
        }
    }

    func deletePriceAlert(priceAlert: PriceAlert) async {
        do {
            try await priceAlertService.delete(priceAlerts: [priceAlert])
        } catch {
            debugLog("deletePriceAlert error: \(error)")
        }
    }
    
    func onSelectSetPriceAlert() {
        isPresentingSetPriceAlert = true
    }
    
    func onSetPriceAlertComplete(message: String) {
        isPresentingSetPriceAlert = false
        isPresentingToastMessage = .priceAlert(message: message)
    }
}
