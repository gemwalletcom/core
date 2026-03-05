// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Testing
import StakeService
import StakeServiceTestKit
import StakeTestKit
import PrimitivesTestKit
import Primitives

@testable import Store
@testable import Stake

@MainActor
struct StakeSceneViewModelTests {

    @Test
    func testAprValue() throws {
        #expect(StakeSceneViewModel.mock(stakeService: MockStakeService(stakeApr: 13.5)).stakeAprModel.subtitle.text == "13.50%")
        #expect(StakeSceneViewModel.mock(stakeService: MockStakeService(stakeApr: 0)).stakeAprModel.subtitle.text == .empty)
        #expect(StakeSceneViewModel.mock(stakeService: MockStakeService(stakeApr: .none)).stakeAprModel.subtitle.text == .empty)
    }
    
    @Test
    func testLockTimeValue() throws {
        #expect(StakeSceneViewModel.mock(chain: .tron).lockTimeValue == "14 days")
    }
    
    @Test
    func minimumStakeAmount() throws {
        #expect(StakeSceneViewModel.mock(chain: .tron).minAmountValue == "1.00 TRX")
    }
    
    @Test
    func showManage() throws {
        #expect(StakeSceneViewModel.mock(wallet: .mock(type: .multicoin)).showManage == true)
        #expect(StakeSceneViewModel.mock(wallet: .mock(type: .view)).showManage == false)
    }

    @Test
    func recommendedCurrentValidator() throws {
        let model = StakeSceneViewModel.mock(chain: .cosmos)
        let recommendedId = try #require(StakeRecommendedValidators().validatorsSet(chain: .cosmos).first)

        model.validatorsQuery.value = [.mock(.cosmos, id: "other"), .mock(.cosmos, id: recommendedId)]

        #expect(model.recommendedCurrentValidator?.id == recommendedId)
    }
}
