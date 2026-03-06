// Copyright (c). Gem Wallet. All rights reserved.

import Testing
import Primitives
import PrimitivesTestKit
import PerpetualsTestKit
import Formatters
import Localization
@testable import Perpetuals

struct CandleTooltipViewModelTests {

    @Test
    func tooltipContent() {
        let model = CandleTooltipViewModel.mock(candle: .mock(open: 67_715, high: 68_181, low: 67_714, close: 68_087, volume: 500))

        #expect(model.openField.title.text == Localized.Charts.Price.open)
        #expect(model.openField.value.text == "67,715.00")

        #expect(model.highField.title.text == Localized.Charts.Price.high)
        #expect(model.highField.value.text == "68,181.00")

        #expect(model.lowField.title.text == Localized.Charts.Price.low)
        #expect(model.lowField.value.text == "67,714.00")

        #expect(model.closeField.title.text == Localized.Charts.Price.close)
        #expect(model.closeField.value.text == "68,087.00")

        #expect(model.changeField.title.text == Localized.Charts.Price.change)
        #expect(model.changeField.value.text == "+0.55%")

        #expect(model.volumeField.title.text == Localized.Perpetual.volume)
        #expect(model.volumeField.value.text == "$34.04M")
    }

    @Test
    func changeSign() {
        #expect(CandleTooltipViewModel.mock(candle: .mock(open: 100, close: 105)).changeField.value.text == "+5.00%")
        #expect(CandleTooltipViewModel.mock(candle: .mock(open: 100, close: 95)).changeField.value.text == "-5.00%")
        #expect(CandleTooltipViewModel.mock(candle: .mock(open: 100, close: 100)).changeField.value.text == "+0.00%")
    }
}
