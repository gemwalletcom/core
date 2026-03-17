// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import Components
import Store
import Localization
import PrimitivesComponents
import PriceService
import Preferences
import PriceAlertService
import SwiftUI
import Formatters
import InfoSheet

@MainActor
@Observable
public final class ChartSceneViewModel: ChartListViewable {
    private let service: ChartService
    private let priceService: PriceService
    private let preferences: Preferences = .standard

    let walletId: WalletId
    let assetModel: AssetViewModel
    let priceAlertService: PriceAlertService

    public var chartState: StateViewType<ChartValuesViewModel> = .loading
    public var selectedPeriod: ChartPeriod

    public let priceQuery: ObservableQuery<PriceRequest>
    var priceData: PriceData? { priceQuery.value }

    var isPresentingInfoSheet: InfoSheetType?
    private let onSetPriceAlert: (Asset) -> Void

    var title: String { assetModel.name }

    var priceAlertsViewModel: PriceAlertsViewModel { PriceAlertsViewModel(priceAlerts: priceData?.priceAlerts ?? []) }
    var showPriceAlerts: Bool { priceAlertsViewModel.hasPriceAlerts && isPriceAvailable }
    var isPriceAvailable: Bool { PriceViewModel(price: priceData?.price, currencyCode: preferences.currency).isPriceAvailable }

    public init(
        service: ChartService = ChartService(),
        priceService: PriceService,
        assetModel: AssetViewModel,
        priceAlertService: PriceAlertService,
        walletId: WalletId,
        currentPeriod: ChartPeriod = ChartValuesViewModel.defaultPeriod,
        onSetPriceAlert: @escaping (Asset) -> Void
    ) {
        self.service = service
        self.priceService = priceService
        self.assetModel = assetModel
        self.priceAlertService = priceAlertService
        self.walletId = walletId
        self.selectedPeriod = currentPeriod
        self.priceQuery = ObservableQuery(PriceRequest(assetId: assetModel.asset.id), initialValue: nil)
        self.onSetPriceAlert = onSetPriceAlert
    }
    
    var priceDataModel: AssetDetailsInfoViewModel? {
        guard let priceData else { return nil }
        return AssetDetailsInfoViewModel(priceData: priceData)
    }
}

// MARK: - Business Logic

extension ChartSceneViewModel {
    public func fetch() async {
        chartState = .loading
        do {
            let values = try await service.getCharts(
                assetId: assetModel.asset.id,
                period: selectedPeriod
            )
            if let market = values.market {
                try priceService.updateMarketPrice(assetId: assetModel.asset.id, market: market, currency: preferences.currency)
            }
            let price = try priceService.getPrice(for: assetModel.asset.id)
            let rate = try priceService.getRate(currency: preferences.currency)
            
            var charts = values.prices.map {
                ChartDateValue(date: Date(timeIntervalSince1970: TimeInterval($0.timestamp)), value: Double($0.value) * rate)
            }

            if let price = price, let last = charts.last, price.updatedAt > last.date {
                charts.append(ChartDateValue(date: .now, value: price.price))
            }

            let chartValues = try ChartValues.from(charts: charts)
            let formatter = CurrencyFormatter(currencyCode: preferences.currency)
            let model = ChartValuesViewModel(
                period: selectedPeriod,
                price: price?.mapToPrice(),
                values: chartValues,
                formatter: formatter
            )
            chartState = .data(model)
        } catch {
            chartState = .error(error)
        }
    }

    public func onSelectSetPriceAlerts() {
        onSetPriceAlert(assetModel.asset)
    }
}
