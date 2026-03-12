// Copyright (c). Gem Wallet. All rights reserved.

import Testing
import Foundation
import Primitives
import PrimitivesTestKit
import PrimitivesComponentsTestKit
@testable import PrimitivesComponents

struct ChartValuesViewModelTests {

    @Test
    func chartValuesViewModel() {
        let model = ChartValuesViewModel.mock(price: .mock(price: 150), values: .mock(values: [100, 200]))

        #expect(model.lowerBoundValueText == "$100.00")
        #expect(model.upperBoundValueText == "$200.00")
        #expect(model.chartHeaderViewModel?.price == 150)
        #expect(model.headerViewModel(for: ChartDateValue(date: Date(), value: 150)).priceChangePercentage == 50)

        #expect(ChartValuesViewModel.mock(price: nil).chartHeaderViewModel == nil)
        #expect(ChartValuesViewModel.mock(period: .week, price: .mock(price: 150), values: .mock(values: [100, 200])).chartHeaderViewModel?.priceChangePercentage == 50)
    }

    @Test
    func headerValue() {
        let model = ChartValuesViewModel.mock(price: .mock(price: 150), values: .mock(values: [100, 200]), headerValue: 500)

        #expect(model.chartHeaderViewModel?.headerValue == 500)
        #expect(model.headerViewModel(for: ChartDateValue(date: Date(), value: 150)).headerValue == 150)
        #expect(ChartValuesViewModel.mock().headerViewModel(for: ChartDateValue(date: Date(), value: 150)).headerValue == nil)
    }
}
