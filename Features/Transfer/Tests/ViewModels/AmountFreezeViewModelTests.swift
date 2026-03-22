// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Testing
import Primitives
import PrimitivesTestKit

@testable import Transfer

struct AmountFreezeViewModelTests {

    @Test
    func title() {
        #expect(AmountFreezeViewModel(asset: .mockTron(), action: .freeze, resource: .bandwidth).title == "Freeze")
        #expect(AmountFreezeViewModel(asset: .mockTron(), action: .unfreeze, resource: .bandwidth).title == "Unfreeze")
    }

    @Test
    func resourceSelection() {
        let model = AmountFreezeViewModel(asset: .mockTron(), action: .freeze, resource: .energy)

        #expect(model.resourceSelection.options == [.bandwidth, .energy])
        #expect(model.resourceSelection.selected == .energy)
        #expect(model.resourceSelection.isEnabled == true)
    }

    @Test
    func minimumValue() {
        #expect(AmountFreezeViewModel(asset: .mockTron(), action: .freeze, resource: .bandwidth).minimumValue > .zero)
        #expect(AmountFreezeViewModel(asset: .mockTron(), action: .unfreeze, resource: .bandwidth).minimumValue == .zero)
    }

    @Test
    func reserveForFee() {
        #expect(AmountFreezeViewModel(asset: .mockTron(), action: .freeze, resource: .bandwidth).reserveForFee > .zero)
        #expect(AmountFreezeViewModel(asset: .mockTron(), action: .unfreeze, resource: .bandwidth).reserveForFee == .zero)
    }

    @Test
    func availableValue() {
        let assetData = AssetData.mock(asset: .mockTron(), balance: .mock(available: 1000, frozen: 500, locked: 300))

        let freeze = AmountFreezeViewModel(asset: .mockTron(), action: .freeze, resource: .bandwidth)
        let unfreezeBandwidth = AmountFreezeViewModel(asset: .mockTron(), action: .unfreeze, resource: .bandwidth)
        let unfreezeEnergy = AmountFreezeViewModel(asset: .mockTron(), action: .unfreeze, resource: .energy)

        #expect(freeze.availableValue(from: assetData) == 1000)
        #expect(unfreezeBandwidth.availableValue(from: assetData) == 500)
        #expect(unfreezeEnergy.availableValue(from: assetData) == 300)
    }

    @Test
    func shouldReserveFee() {
        let assetData = AssetData.mock(asset: .mockTron(), balance: .mock(available: 10_000_000_000))

        let freeze = AmountFreezeViewModel(asset: .mockTron(), action: .freeze, resource: .bandwidth)
        let unfreeze = AmountFreezeViewModel(asset: .mockTron(), action: .unfreeze, resource: .bandwidth)

        #expect(freeze.shouldReserveFee(from: assetData) == true)
        #expect(unfreeze.shouldReserveFee(from: assetData) == false)
    }

    @Test
    func recipientData() {
        let model = AmountFreezeViewModel(asset: .mockTron(), action: .freeze, resource: .energy)
        model.resourceSelection.selected = .bandwidth

        #expect(model.recipientData().recipient.name == "Bandwidth")
    }

    @Test
    func makeTransferData() throws {
        let freeze = AmountFreezeViewModel(asset: .mockTron(), action: .freeze, resource: .bandwidth)
        let unfreeze = AmountFreezeViewModel(asset: .mockTron(), action: .unfreeze, resource: .energy)

        let freezeData = try freeze.makeTransferData(value: 100)
        let unfreezeData = try unfreeze.makeTransferData(value: 200)

        #expect(freezeData.type.transactionType == .stakeFreeze)
        #expect(unfreezeData.type.transactionType == .stakeUnfreeze)
        #expect(freezeData.value == 100)
        #expect(unfreezeData.value == 200)
    }

    @Test
    func makeTransferDataUsesSelectedResource() throws {
        let model = AmountFreezeViewModel(asset: .mockTron(), action: .freeze, resource: .bandwidth)
        model.resourceSelection.selected = .energy

        let transferData = try model.makeTransferData(value: 100)

        #expect(transferData.type.metadata?.decode(TransactionResourceTypeMetadata.self)?.resourceType == .energy)
    }
}
