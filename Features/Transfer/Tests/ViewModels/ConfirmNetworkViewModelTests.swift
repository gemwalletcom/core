// Copyright (c). Gem Wallet. All rights reserved.

import Testing
import Localization
@testable import Transfer
@testable import Primitives
import PrimitivesTestKit
import TransferTestKit

struct ConfirmNetworkViewModelTests {

    @Test
    func transfer() {
        let asset = Asset.mock()
        let model = ConfirmNetworkViewModel(type: .transfer(asset))

        guard case .network(let item) = model.itemModel else { return }
        #expect(item.title == Localized.Transfer.network)
        #expect(item.subtitle != nil)
        #expect(item.imageStyle != nil)
    }
}
