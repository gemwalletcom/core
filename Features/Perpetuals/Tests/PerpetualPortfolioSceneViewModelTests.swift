// Copyright (c). Gem Wallet. All rights reserved.

import Testing
import Primitives
import PrimitivesTestKit
import PrimitivesComponents
import Components
import PerpetualService
import PerpetualServiceTestKit
import PerpetualsTestKit
@testable import Perpetuals

struct PerpetualPortfolioSceneViewModelTests {

    @Test
    @MainActor
    func navigationTitle() {
        #expect(PerpetualPortfolioSceneViewModel.mock().navigationTitle == "Perpetuals")
    }

    @Test
    @MainActor
    func chartTypeTitle() {
        let model = PerpetualPortfolioSceneViewModel.mock()
        #expect(model.chartTypeTitle(.value) == "Value")
        #expect(model.chartTypeTitle(.pnl) == "PnL")
    }

    @Test
    @MainActor
    func periods() {
        let model = PerpetualPortfolioSceneViewModel.mock()
        #expect(model.periods == [.day, .week, .month, .all])

        model.state = .data(.mock(day: .mock(), week: .mock(), month: nil, allTime: nil))
        #expect(model.periods == [.day, .week])
    }

    @Test
    @MainActor
    func chartState() {
        let model = PerpetualPortfolioSceneViewModel.mock()
        model.selectedChartType = .value

        model.state = .loading
        #expect(model.chartState.isLoading)

        model.state = .noData
        #expect(model.chartState.isNoData)

        model.state = .error(AnyError("test"))
        #expect(model.chartState.isError)

        model.state = .data(.mock(day: .mock(accountValueHistory: ChartDateValue.mockHistory(values: [100, 100, 100]))))
        #expect(model.chartState.isNoData)

        model.state = .data(.mock(day: .mock(accountValueHistory: ChartDateValue.mockHistory(values: [100, 105, 110]))))
        #expect(model.chartState.value != nil)
    }

    @Test
    @MainActor
    func chartStateType() {
        let model = PerpetualPortfolioSceneViewModel.mock()
        model.state = .data(.mock(day: .mock(
            accountValueHistory: ChartDateValue.mockHistory(values: [100, 110]),
            pnlHistory: ChartDateValue.mockHistory(values: [0, 10])
        )))

        model.selectedChartType = .value
        if case .data(let chartModel) = model.chartState {
            #expect(chartModel.type == .priceChange)
        }

        model.selectedChartType = .pnl
        if case .data(let chartModel) = model.chartState {
            #expect(chartModel.type == .priceChange)
        }
    }

    @Test
    @MainActor
    func marginUsageField() {
        let model = PerpetualPortfolioSceneViewModel.mock()

        #expect(model.marginUsageField.value.text == "-")

        model.state = .data(.mock(accountSummary: .mock(accountValue: 100, marginUsage: 0.168)))
        #expect(model.marginUsageField.value.text == "$16.80 (16.80%)")

        model.state = .data(.mock(accountSummary: .mock(accountValue: 0, marginUsage: 0)))
        #expect(model.marginUsageField.value.text == "$0.00 (0.00%)")
    }

    @Test
    @MainActor
    func valueChangeCalculation() {
        let model = PerpetualPortfolioSceneViewModel.mock()
        model.selectedChartType = .value
        model.state = .data(.mock(day: .mock(accountValueHistory: ChartDateValue.mockHistory(values: [0, 50, 30, 100]))))

        if case .data(let chartModel) = model.chartState {
            #expect(chartModel.price?.price == 100)
            #expect(chartModel.price?.priceChangePercentage24h == 0)
        }

        model.state = .data(.mock(day: .mock(accountValueHistory: ChartDateValue.mockHistory(values: [50, 100, 75]))))
        if case .data(let chartModel) = model.chartState {
            #expect(chartModel.price?.price == 25)
            #expect(chartModel.price?.priceChangePercentage24h == 50)
        }
    }
}

extension PerpetualPortfolioSceneViewModel {
    @MainActor
    static func mock(
        wallet: Wallet = .mock(),
        perpetualService: PerpetualServiceable = PerpetualService.mock()
    ) -> PerpetualPortfolioSceneViewModel {
        PerpetualPortfolioSceneViewModel(
            wallet: wallet,
            perpetualService: perpetualService
        )
    }
}
