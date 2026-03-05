// Copyright (c). Gem Wallet. All rights reserved.

import Primitives
import PrimitivesTestKit
import StakeService
import StakeServiceTestKit
@testable import Stake

public extension StakeSceneViewModel {
    static func mock(
        wallet: Wallet = .mock(),
        chain: StakeChain = .tron,
        stakeService: any StakeServiceable = MockStakeService(stakeApr: 13.5)
    ) -> StakeSceneViewModel {
        StakeSceneViewModel(
            wallet: wallet,
            chain: chain,
            currencyCode: "USD",
            stakeService: stakeService
        )
    }
}
