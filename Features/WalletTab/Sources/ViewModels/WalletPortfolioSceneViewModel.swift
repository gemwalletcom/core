// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import Components
import Formatters
import PrimitivesComponents
import PriceService
import Store

@Observable
@MainActor
public final class WalletPortfolioSceneViewModel: ChartListViewable {
    private let wallet: Wallet

    private let service: PortfolioService
    private let priceService: PriceService

    private let currencyCode: String
    private let currencyFormatter: CurrencyFormatter
    private let priceFormatter: CurrencyFormatter
    private let percentFormatter: CurrencyFormatter

    public let assetsQuery: ObservableQuery<AssetsRequest>

    private var assets: [AssetData] { assetsQuery.value }

    var state: StateViewType<WalletPortfolioData> = .loading
    public var selectedPeriod: ChartPeriod = .day

    public init(
        wallet: Wallet,
        portfolioService: PortfolioService,
        priceService: PriceService,
        currencyCode: String
    ) {
        self.service = portfolioService
        self.priceService = priceService
        self.wallet = wallet

        self.currencyCode = currencyCode
        self.currencyFormatter = CurrencyFormatter(type: .currency, currencyCode: currencyCode)
        self.priceFormatter = CurrencyFormatter(currencyCode: currencyCode)
        self.percentFormatter = CurrencyFormatter(type: .percent, currencyCode: currencyCode)

        self.assetsQuery = ObservableQuery(AssetsRequest(walletId: wallet.walletId, filters: [.enabledBalance]), initialValue: [])
    }

    var navigationTitle: String { wallet.name }

    public var periods: [ChartPeriod] { [.day, .week, .month, .year, .all] }
    public var chartState: StateViewType<ChartValuesViewModel> { state.map { $0.chart } }

    var allTimeValues: [ListItemModel] {
        guard case .data(let data) = state else { return [] }
        let allTime = AllTimeValueViewModel(priceFormatter: priceFormatter, percentFormatter: percentFormatter)
        return [
            data.portfolio.allTimeHigh.map { allTime.allTimeHigh(chartValue: $0) },
            data.portfolio.allTimeLow.map { allTime.allTimeLow(chartValue: $0) },
        ].compactMap { $0 }
    }
}

// MARK: - Business Logic

extension WalletPortfolioSceneViewModel {
    public func fetch() async {
        guard !assets.isEmpty else { return }
        state = .loading
        do {
            let rate = try priceService.getRate(currency: currencyCode)
            let portfolio = try await service.getPortfolioAssets(assets: assets, period: selectedPeriod)
            let charts = portfolio.values.map {
                ChartDateValue(date: Date(timeIntervalSince1970: TimeInterval($0.timestamp)), value: Double($0.value) * rate)
            }
            let chart = ChartValuesViewModel.priceChange(charts: charts, period: selectedPeriod, formatter: currencyFormatter, showHeaderValue: true)
            state = chart.map { .data(WalletPortfolioData(chart: $0, portfolio: portfolio)) } ?? .noData
        } catch {
            state = .error(error)
        }
    }
}
