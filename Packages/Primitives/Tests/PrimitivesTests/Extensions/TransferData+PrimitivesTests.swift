// Copyright (c). Gem Wallet. All rights reserved.

import Testing
import Primitives
import PrimitivesTestKit

struct TransferDataTypeTests {
    @Test
    func shouldIgnoreValueCheck() {
        #expect(TransferData.mock(type: .transferNft(.mock())).type.shouldIgnoreValueCheck == true)
        #expect(TransferData.mock(type: .stake(.mock(), .stake(.mock()))).type.shouldIgnoreValueCheck == true)
        #expect(TransferData.mock(type: .account(.mock(), .activate)).type.shouldIgnoreValueCheck == true)
        #expect(TransferData.mock(type: .transfer(.mock())).type.shouldIgnoreValueCheck == false)

        #expect(TransferData.mock(type: .deposit(.mock())).type.shouldIgnoreValueCheck == false)
        #expect(TransferData.mock(type: .perpetual(.mock(), .open(.mock(direction: .long, assetIndex: 0, price: "100", size: "1")))).type.shouldIgnoreValueCheck == true)
        #expect(
            TransferData
                .mock(
                    type: .perpetual(.mock(), .close(.mock(direction: .long, assetIndex: 0, price: "100", size: "1")))
                ).type.shouldIgnoreValueCheck == true
        )
    }
    // MARK: - canChangeValue

    @Test
    func canChangeValue() {
        #expect(TransferData.mock(type: .transfer(.mock())).canChangeValue == true)
        #expect(TransferData.mock(type: .transfer(.mock()), canChangeValue: false).canChangeValue == false)

        #expect(TransferData.mock(type: .swap(.mock(), .mock(), .mock())).canChangeValue == true)

        #expect(TransferData.mock(type: .stake(.mock(), .stake(.mock()))).canChangeValue == true)
        #expect(TransferData.mock(type: .stake(.mock(), .redelegate(.mock()))).canChangeValue == true)

        #expect(TransferData.mock(type: .stake(.mock(), .unstake(.mock(state: .inactive)))).canChangeValue == true)
        #expect(TransferData.mock(type: .stake(.mock(), .withdraw(.mock(state: .inactive)))).canChangeValue == true)
        #expect(TransferData.mock(type: .stake(.mock(), .rewards([.mock()]))).canChangeValue == true)

        #expect(TransferData.mock(type: .transferNft(.mock())).canChangeValue == true)
        #expect(TransferData.mock(type: .tokenApprove(.mock(), .mock())).canChangeValue == true)
        #expect(TransferData.mock(type: .account(.mock(), .activate)).canChangeValue == true)
        #expect(TransferData.mock(type: .generic(asset: .mock(), metadata: .mock(), extra: .mock(outputType: .encodedTransaction))).canChangeValue == true)

        #expect(TransferData.mock(type: .deposit(.mock())).canChangeValue == true)
        #expect(TransferData.mock(type: .deposit(.mock()), canChangeValue: false).canChangeValue == false)
        #expect(TransferData.mock(type: .perpetual(.mock(), .open(.mock(direction: .long, assetIndex: 0, price: "100", size: "1")))).canChangeValue == true)
        #expect(
            TransferData
                .mock(
                    type: .perpetual(.mock(), .close(.mock(direction: .long, assetIndex: 0, price: "100", size: "1"))),
                    canChangeValue: false
                ).canChangeValue == false
        )
    }

    @Test
    func perpetualOpenTransactionType() {
        let asset = Asset.mock()

        let openType = TransferDataType.perpetual(asset, .mockOpen())
        let increaseType = TransferDataType.perpetual(asset, .mockIncrease())

        #expect(openType.transactionType == .perpetualOpenPosition)
        #expect(increaseType.transactionType == .perpetualOpenPosition)
    }

    @Test
    func perpetualCloseTransactionType() {
        let asset = Asset.mock()

        let closeType = TransferDataType.perpetual(asset, .mockClose())
        let reduceType = TransferDataType.perpetual(asset, .mockReduce())

        #expect(closeType.transactionType == .perpetualClosePosition)
        #expect(reduceType.transactionType == .perpetualClosePosition)
    }

    @Test
    func perpetualModifyTransactionType() {
        let asset = Asset.mock()
        let modifyType = TransferDataType.perpetual(asset, .mockModify())

        #expect(modifyType.transactionType == .perpetualModifyPosition)
    }

    @Test
    func withGasLimit() throws {
        let type = TransferDataType.swap(.mock(), .mock(), .mock(data: SwapQuoteData(to: "", dataType: .contract, value: "", data: "", memo: nil, approval: nil, gasLimit: "0")))

        let (_, _, swapBefore) = try type.swap()
        #expect(swapBefore.data.gasLimit == "0")

        let (_, _, swapAfter) = try type.withGasLimit("21000").swap()
        #expect(swapAfter.data.gasLimit == "21000")
    }

    @Test
    func freezeMetadata() {
        let bandwidth = TransferDataType.stake(.mock(), .freeze(.bandwidth))
        let energy = TransferDataType.stake(.mock(), .unfreeze(.energy))

        #expect(bandwidth.metadata == .object(["resourceType": .string("bandwidth")]))
        #expect(energy.metadata == .object(["resourceType": .string("energy")]))
    }
}
