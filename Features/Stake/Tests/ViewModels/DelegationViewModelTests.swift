// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Testing
import Stake
import PrimitivesTestKit
import Primitives

struct DelegationViewModelTests {

    @Test
    func balance() {
        let model = DelegationViewModel.mock()

        #expect(model.balanceText == "1,500.00 TRX")
        #expect(model.fiatValueText == "$3,000.00")
    }

    @Test
    func rewards() {
        let model = DelegationViewModel.mock()

        #expect(model.rewardsText == "500.00 TRX")
        #expect(model.rewardsFiatValueText == "$1,000.00")
    }

    @Test
    func completionDate() {
        #expect(
            DelegationViewModel
                .mock(state: .deactivating, completionDate: Date.now.addingTimeInterval(86400))
                .completionDateText == "23 hours, 59 minutes"
        )
    }
}

extension DelegationViewModel {
    static func mock(
        state: DelegationState = .active,
        completionDate: Date? = nil
    ) -> DelegationViewModel {
        DelegationViewModel(
            delegation: .mock(
                state: state,
                price: Price.mock(price: 2.0),
                base: .mock(
                    state: state,
                    assetId: .mock(.tron),
                    balance: "1500000000",
                    rewards: "500000000",
                    completionDate: completionDate
                )
            ),
            asset: Chain.tron.asset,
            currencyCode: "USD"
        )
    }
}
