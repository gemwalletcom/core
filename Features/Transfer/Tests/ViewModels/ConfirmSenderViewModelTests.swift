// Copyright (c). Gem Wallet. All rights reserved.

import Testing
import Localization
@testable import Transfer
@testable import Primitives
import PrimitivesTestKit
import TransferTestKit

struct ConfirmSenderViewModelTests {

    @Test
    func wallet() {
        let wallet = Wallet.mock()
        let model = ConfirmSenderViewModel(wallet: wallet)

        guard case .sender(let item) = model.itemModel else { return }
        #expect(item.title == Localized.Common.wallet)
        #expect(item.subtitle == wallet.name)
        #expect(item.imageStyle != nil)
    }
}
