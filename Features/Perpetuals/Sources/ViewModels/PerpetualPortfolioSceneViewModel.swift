// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import Components
import Formatters
import PerpetualService
import PrimitivesComponents
import Localization

@Observable
@MainActor
final class PerpetualPortfolioSceneViewModel: ChartListViewable {
    private let wallet: Wallet
    private let perpetualService: PerpetualServiceable
    private let currencyFormatter = CurrencyFormatter(type: .currency, currencyCode: Currency.usd.rawValue)

    var state: StateViewType<PerpetualPortfolio> = .loading {
        didSet {
            switch state {
            case .data(let data): portfolio = data
            case .error: portfolio = nil
            case .loading, .noData: break
            }
        }
    }
    public var selectedPeriod: ChartPeriod = .day
    var selectedChartType: PerpetualPortfolioChartType = .pnl

    private var portfolio: PerpetualPortfolio?

    init(
        wallet: Wallet,
        perpetualService: PerpetualServiceable
    ) {
        self.wallet = wallet
        self.perpetualService = perpetualService
    }

    var navigationTitle: String { Localized.Perpetuals.title }
    var infoSectionTitle: String { Localized.Common.info }

    public var periods: [ChartPeriod] {
        guard let periods = portfolio?.availablePeriods, !periods.isEmpty else {
            return [.day, .week, .month, .all]
        }
        return periods
    }

    public var chartState: StateViewType<ChartValuesViewModel> {
        switch state {
        case .loading: .loading
        case .noData: .noData
        case .error(let error): .error(error)
        case .data(let data): chartModel(data: data).map { .data($0) } ?? .noData
        }
    }

    func chartTypeTitle(_ type: PerpetualPortfolioChartType) -> String {
        switch type {
        case .value: Localized.Perpetual.value
        case .pnl: Localized.Perpetual.pnl
        }
    }

    public func fetch() async {
        guard let address = wallet.hyperliquidAccount?.address else { return }
        state = .loading
        do {
            let data = try await perpetualService.portfolio(address: address)
            if !data.availablePeriods.contains(selectedPeriod), let first = data.availablePeriods.first {
                selectedPeriod = first
            }
            state = .data(data)
        } catch {
            state = .error(error)
        }
    }
}

// MARK: - Stats

extension PerpetualPortfolioSceneViewModel {
    var unrealizedPnlField: ListItemField {
        ListItemField(
            title: TextValue(text: Localized.Perpetual.unrealizedPnl, style: ListItemModel.StyleDefaults.titleStyle),
            value: TextValue(text: unrealizedPnlModel.text ?? "-", style: unrealizedPnlModel.textStyle)
        )
    }

    var accountLeverageField: ListItemField {
        ListItemField(title: Localized.Perpetual.accountLeverage, value: accountLeverageText)
    }

    var marginUsageField: ListItemField {
        ListItemField(title: Localized.Perpetual.marginUsage, value: marginUsageText)
    }

    var allTimePnlField: ListItemField {
        ListItemField(
            title: TextValue(text: Localized.Perpetual.allTimePnl, style: ListItemModel.StyleDefaults.titleStyle),
            value: TextValue(text: allTimePnlModel.text ?? "-", style: allTimePnlModel.textStyle)
        )
    }

    var volumeField: ListItemField {
        ListItemField(title: Localized.Perpetual.volume, value: volumeText)
    }
}

// MARK: - Private

extension PerpetualPortfolioSceneViewModel {
    private var unrealizedPnlModel: PriceChangeViewModel { priceChangeModel(value: portfolio?.accountSummary?.unrealizedPnl) }
    private var allTimePnlModel: PriceChangeViewModel { priceChangeModel(value: portfolio?.allTime?.pnlHistory.last?.value) }

    private var accountLeverageText: String { portfolio?.accountSummary.map { String(format: "%.2fx", $0.accountLeverage) } ?? "-" }
    private var marginUsageText: String {
        portfolio?.accountSummary.map {
            let marginValue = currencyFormatter.string($0.accountValue * $0.marginUsage)
            let marginPercent = CurrencyFormatter.percentSignLess.string($0.marginUsage * 100)
            return "\(marginValue) (\(marginPercent))"
        } ?? "-"
    }
    private var volumeText: String { portfolio.map { currencyFormatter.string($0.allTime?.volume ?? 0) } ?? "-" }

    private func priceChangeModel(value: Double?) -> PriceChangeViewModel {
        PriceChangeViewModel(value: value, currencyFormatter: currencyFormatter)
    }

    private func chartModel(data: PerpetualPortfolio) -> ChartValuesViewModel? {
        guard let timeframe = data.timeframeData(for: selectedPeriod) else {
            return nil
        }
        let charts: [ChartDateValue] = switch selectedChartType {
        case .value:
            Array(timeframe.accountValueHistory.drop(while: { $0.value == .zero }))
        case .pnl: timeframe.pnlHistory
        }
        return .priceChange(charts: charts, period: selectedPeriod, formatter: currencyFormatter, showHeaderValue: selectedChartType == .value)
    }
}
