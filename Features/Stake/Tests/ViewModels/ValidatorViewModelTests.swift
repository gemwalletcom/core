// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Testing
import Stake
import PrimitivesTestKit

struct ValidatorViewModelTests {

    @Test func aprText() {
        let model = ValidatorViewModel(validator: .mock(apr: 2.15))

        #expect(model.aprModel.text == "APR 2.15%")
    }
}
