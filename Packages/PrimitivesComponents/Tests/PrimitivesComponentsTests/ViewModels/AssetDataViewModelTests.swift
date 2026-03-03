// Copyright (c). Gem Wallet. All rights reserved.

import Testing
import BigInt
import Primitives
import PrimitivesTestKit
import PrimitivesComponentsTestKit

@testable import PrimitivesComponents

struct AssetDataViewModelTests {

    @Test
    func apr() {
        let model = AssetDataViewModel.mock(assetData: .mock(metadata: .mock(stakingApr: 5.0, earnApr: 3.0)))

        #expect(model.apr(for: .stake) == 5.0)
        #expect(model.apr(for: .earn) == 3.0)
        #expect(AssetDataViewModel.mock().apr(for: .stake) == nil)
        #expect(AssetDataViewModel.mock().apr(for: .earn) == nil)
    }

    @Test
    func balanceTextWithSymbol() {
        let model = AssetDataViewModel.mock(assetData: .mock(
            asset: .mockEthereum(),
            balance: .mock(staked: BigInt(1_000_000_000_000_000_000), earn: BigInt(2_000_000_000_000_000_000))
        ))

        #expect(model.balanceTextWithSymbol(for: .stake) == "1.00 ETH")
        #expect(model.balanceTextWithSymbol(for: .earn) == "2.00 ETH")
    }
}
