// Copyright (c). Gem Wallet. All rights reserved.

import Testing
import BigInt
import Localization
import Primitives
import PrimitivesTestKit
@testable import PrimitivesComponents

struct AssetValueHeaderViewModelTests {

    @Test
    func unlimitedTitle() {
        let model = AssetValueHeaderViewModel(
            data: AssetValueHeaderData(asset: .mockEthereumUSDT(), value: .unlimited)
        )

        #expect(model.title == Localized.Simulation.Header.unlimitedAsset("USDT"))
        #expect(model.subtitle == nil)
    }

    @Test
    func formattedNumericTitle() {
        let model = AssetValueHeaderViewModel(
            data: AssetValueHeaderData(asset: .mockEthereumUSDT(), value: .exact(BigInt(1000000)))
        )

        #expect(model.title == "1 USDT")
    }
}
