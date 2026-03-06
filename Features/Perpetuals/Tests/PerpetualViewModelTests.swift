// Copyright (c). Gem Wallet. All rights reserved.

import Testing
import Primitives
import PrimitivesTestKit
import PerpetualsTestKit
@testable import Perpetuals

struct PerpetualViewModelTests {
    
    @Test
    func name() {
        #expect(PerpetualViewModel(perpetual: .mock(name: "BTC-PERP")).name == "BTC-PERP")
    }
    
    @Test
    func volumeField() {
        #expect(PerpetualViewModel(perpetual: .mock(volume24h: 1_500_000)).volumeField.value.text == "$1.5M")
    }

    @Test
    func openInterestField() {
        #expect(PerpetualViewModel(perpetual: .mock(openInterest: 5_250_000)).openInterestField.value.text == "$5.25M")
    }

    @Test
    func fundingRateField() {
        #expect(PerpetualViewModel(perpetual: .mock(funding: 0.0001)).fundingRateField.value.text.contains("%"))
    }
    
    @Test
    func priceText() {
        #expect(PerpetualViewModel(perpetual: .mock(price: 45_000)).priceText == "$45,000.00")
        #expect(PerpetualViewModel(perpetual: .mock(price: 0.5)).priceText == "$0.5")
        #expect(PerpetualViewModel(perpetual: .mock(price: 1_234.56)).priceText == "$1,234.56")
    }
}
