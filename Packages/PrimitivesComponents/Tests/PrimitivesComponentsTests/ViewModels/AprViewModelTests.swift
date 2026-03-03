// Copyright (c). Gem Wallet. All rights reserved.

import Testing
import PrimitivesComponents

struct AprViewModelTests {

    @Test
    func subtitle() {
        #expect(AprViewModel(apr: 13.5).subtitle.text == "13.50%")
        #expect(AprViewModel(apr: .zero).subtitle.text == .empty)
    }

    @Test
    func text() {
        #expect(AprViewModel(apr: 2.15).text == "APR 2.15%")
        #expect(AprViewModel(apr: .zero).text == "APR ")
    }

    @Test
    func showApr() {
        #expect(AprViewModel(apr: 5.0).showApr == true)
        #expect(AprViewModel(apr: .zero).showApr == false)
    }
}
