// Copyright (c). Gem Wallet. All rights reserved.

import Testing
import Stake

struct StakeRecommendedValidatorsTests {

    @Test func testValidatorsSet() async throws {
        let model = StakeRecommendedValidators()
        
        #expect(model.validatorsSet(chain: .bitcoin).isEmpty)
        #expect(model.validatorsSet(chain: .cosmos).isEmpty == false)
    }
}
