// Copyright (c). Gem Wallet. All rights reserved.

import Testing
import Localization
import Foundation
@testable import Transfer
@testable import Primitives
import PrimitivesTestKit
import TransferTestKit

struct ConfirmAppViewModelTests {

    @Test
    func generic() {
        let metadata = WalletConnectionSessionAppMetadata.mock(
            name: "PancakeSwap - Trade",
            url: "https://pancakeswap.finance/swap"
        )
        let model = ConfirmAppViewModel(type: .generic(asset: .mock(), metadata: metadata, extra: .mock()))

        guard case .app(let item) = model.itemModel else { return }
        #expect(item.title == Localized.WalletConnect.app)
        #expect(item.subtitle == "PancakeSwap")
        #expect(model.websiteURL == URL(string: metadata.url))
        #expect(model.websiteTitle == Localized.Settings.website)
    }
}
