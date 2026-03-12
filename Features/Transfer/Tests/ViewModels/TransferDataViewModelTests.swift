// Copyright (c). Gem Wallet. All rights reserved.

import Testing
import Localization
@testable import Transfer
@testable import Primitives
import PrimitivesTestKit

struct TransferDataViewModelTests {

    @Test
    func depositTitle() {
        #expect(TransferDataViewModel.mock(type: .deposit(.mock())).title == "Deposit")
    }

    @Test
    func genericSendTitle() {
        let type = TransferDataType.generic(asset: .mock(), metadata: .mock(), extra: .mock(outputAction: .send))
        #expect(TransferDataViewModel.mock(type: type).title == Localized.Transfer.reviewRequest)
    }

    @Test
    func genericSignTitle() {
        let type = TransferDataType.generic(asset: .mock(), metadata: .mock(), extra: .mock(outputAction: .sign))
        #expect(TransferDataViewModel.mock(type: type).title == Localized.Transfer.reviewRequest)
    }
}

private extension TransferDataViewModel {
    static func mock(
        type: TransferDataType = .transfer(.mock())
    ) -> TransferDataViewModel {
        TransferDataViewModel(
            data: TransferData.mock(type: type)
        )
    }
}
