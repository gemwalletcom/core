// Copyright (c). Gem Wallet. All rights reserved.

import Testing
import Foundation
import Primitives
import Style
import PrimitivesComponentsTestKit
@testable import PrimitivesComponents

struct ChartHeaderViewModelTests {

    @Test
    func priceText() {
        #expect(ChartHeaderViewModel.mock(price: 100).priceText == "$100.00")
        #expect(ChartHeaderViewModel.mock(price: 100, type: .priceChange).priceText == "+$100.00")
        #expect(ChartHeaderViewModel.mock(price: -50, type: .priceChange).priceText == "-$50.00")
    }

    @Test
    func priceColor() {
        #expect(ChartHeaderViewModel.mock(price: 100).priceColor == Colors.black)
        #expect(ChartHeaderViewModel.mock(price: 100, type: .priceChange).priceColor == Colors.green)
    }

    @Test
    func priceChangeText() {
        #expect(ChartHeaderViewModel.mock(price: 100, priceChangePercentage: 5.5).priceChangeText == "+5.50%")
        #expect(ChartHeaderViewModel.mock(price: 100, type: .priceChange).priceChangeText == nil)
        #expect(ChartHeaderViewModel.mock(price: 0).priceChangeText == nil)
        #expect(ChartHeaderViewModel.mock(price: 50, priceChangePercentage: 10, headerValue: 200, type: .priceChange).priceChangeText == "(10.00%)")
        #expect(ChartHeaderViewModel.mock(price: 50, priceChangePercentage: 0, headerValue: 200, type: .priceChange).priceChangeText == nil)
    }

    @Test
    func priceChangeTextColor() {
        #expect(ChartHeaderViewModel.mock(priceChangePercentage: 10).priceChangeTextColor == Colors.green)
        #expect(ChartHeaderViewModel.mock(priceChangePercentage: -10).priceChangeTextColor == Colors.red)
    }

    @Test
    func dateText() {
        #expect(ChartHeaderViewModel.mock(date: nil).dateText == nil)
        #expect(ChartHeaderViewModel.mock(date: Date()).dateText != nil)
    }

    @Test
    func headerValueText() {
        #expect(ChartHeaderViewModel.mock().headerValueText == nil)
        #expect(ChartHeaderViewModel.mock(headerValue: 1500).headerValueText == "$1,500.00")
    }

}
