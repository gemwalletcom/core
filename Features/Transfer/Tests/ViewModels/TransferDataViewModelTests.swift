// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
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

    @Test
    func availableValueForUnfreeze() {
        let metadata = TransferDataMetadata(
            assetId: .mock(),
            feeAssetId: .mock(),
            assetBalance: Balance(available: 1000, frozen: 500, locked: 300),
            assetFeeBalance: Balance(available: 1000),
            assetPrices: [:]
        )

        #expect(
            TransferDataViewModel.mock(type: .stake(.mock(), .unfreeze(.bandwidth)))
                .availableValue(metadata: metadata) == BigInt(500)
        )
        #expect(
            TransferDataViewModel.mock(type: .stake(.mock(), .unfreeze(.energy)))
                .availableValue(metadata: metadata) == BigInt(300)
        )
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
