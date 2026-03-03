// Copyright (c). Gem Wallet. All rights reserved.

import Testing
import PrimitivesTestKit
import Primitives
import EarnServiceTestKit

@testable import Transfer
@testable import Store

@MainActor
struct AmountSceneViewModelTests {

    @Test
    func maxButton() {
        let model = AmountSceneViewModel.mock()
        #expect(model.amountInputModel.isValid)

        model.onSelectMaxButton()
        #expect(model.amountInputModel.isValid)

        model.onSelectInputButton()
        model.onSelectMaxButton()
        #expect(model.amountInputModel.isValid)
    }

    @Test
    func stakingReservedFeesText() {
        let assetData = AssetData.mock(
            asset: .mockBNB(),
            balance: .mock(available: 2_000_000_000_000_000_000)
        )
        let model = AmountSceneViewModel.mock(
            type: .stake(.stake(validators: [.mock()], recommended: nil)),
            assetData: assetData
        )

        model.onSelectMaxButton()
        #expect(model.infoText != nil)
        #expect(model.amountInputModel.text == "1.99975")

        model.amountInputModel.text = .zero
        #expect(model.infoText == nil)
    }

    @Test
    func stakeValidation() {
        let assetData = AssetData.mock(
            asset: .mockBNB(),
            balance: .mock(available: 5_000_000_000_000_000_000)
        )
        let model = AmountSceneViewModel.mock(
            type: .stake(.stake(validators: [.mock()], recommended: nil)),
            assetData: assetData
        )

        model.amountInputModel.update(text: "0.099")
        #expect(model.amountInputModel.isValid == false)

        model.amountInputModel.update(text: "1.5")
        #expect(model.amountInputModel.isValid == true)
    }

    @Test
    func transferValidation() {
        let assetData = AssetData.mock(
            asset: .mockBNB(),
            balance: .mock(available: 10_000_000_000_000_000)
        )
        let model = AmountSceneViewModel.mock(
            type: .transfer(recipient: .mock()),
            assetData: assetData
        )

        model.amountInputModel.update(text: "0.001")
        #expect(model.amountInputModel.isValid == true)

        model.amountInputModel.update(text: "100")
        #expect(model.amountInputModel.isValid == false)
    }

    @Test
    func unfreezeResourceSwitch() {
        let assetData = AssetData.mock(
            asset: .mockTron(),
            balance: .mock(frozen: 0, locked: 5_000_000)
        )
        let model = AmountSceneViewModel.mock(
            type: .freeze(data: .init(freezeType: .unfreeze, resource: .bandwidth)),
            assetData: assetData
        )

        guard case let .freeze(freeze) = model.provider else { return }

        freeze.resourceSelection.selected = .energy
        model.onChangeResource(.bandwidth, .energy)
        model.amountInputModel.update(text: "2.0")
        #expect(model.amountInputModel.isValid == true)

        freeze.resourceSelection.selected = .bandwidth
        model.onChangeResource(.energy, .bandwidth)
        model.amountInputModel.update(text: "2.0")
        #expect(model.amountInputModel.isValid == false)
    }

    @Test
    func selectValidatorPreservesAmount() {
        let validator1 = DelegationValidator.mock(id: "1")
        let validator2 = DelegationValidator.mock(id: "2")
        let assetData = AssetData.mock(
            asset: .mockBNB(),
            balance: .mock(available: 5_000_000_000_000_000_000)
        )
        let model = AmountSceneViewModel.mock(
            type: .stake(.stake(validators: [validator1, validator2], recommended: validator1)),
            assetData: assetData
        )

        model.amountInputModel.update(text: "1.5")
        model.onValidatorSelected(validator2)

        #expect(model.amountInputModel.text == "1.5")
    }

    @Test
    func actionButtonState() {
        let model = AmountSceneViewModel.mock()

        #expect(model.actionButtonState == .disabled)

        model.amountInputModel.update(text: "1.0")
        #expect(model.actionButtonState == .normal)

        model.amountInputModel.update(text: "")
        #expect(model.actionButtonState == .disabled)
    }

    @Test
    func onAppearSetsMaxForFixedValue() {
        let delegation = Delegation.mock(base: .mock(state: .active, balance: "1000000"))
        let assetData = AssetData.mock(asset: .mockBNB())
        let model = AmountSceneViewModel.mock(
            type: .stake(.withdraw(delegation)),
            assetData: assetData
        )

        #expect(model.isInputDisabled == true)

        model.onAppear()
        #expect(model.amountInputModel.text.isEmpty == false)
    }
}

extension AmountSceneViewModel {
    static func mock(
        type: AmountType = .transfer(recipient: .mock()),
        assetData: AssetData = .mock(balance: .mock())
    ) -> AmountSceneViewModel {
        let model = AmountSceneViewModel(
            input: AmountInput(type: type, asset: assetData.asset),
            wallet: .mock(),
            service: AmountService(earnDataProvider: MockEarnService()),
            onTransferAction: { _ in }
        )
        model.assetQuery.value = assetData
        model.onChangeAssetBalance(assetData, assetData)
        return model
    }
}
