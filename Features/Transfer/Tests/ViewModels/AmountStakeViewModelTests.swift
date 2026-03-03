// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Testing
import Primitives
import PrimitivesTestKit

@testable import Transfer

struct AmountStakeViewModelTests {

    @Test
    func title() {
        #expect(AmountStakeViewModel(asset: .mockBNB(), action: .stake(validators: [.mock()], recommended: nil)).title == "Stake")
        #expect(AmountStakeViewModel(asset: .mockBNB(), action: .unstake(.mock())).title == "Unstake")
        #expect(AmountStakeViewModel(asset: .mockBNB(), action: .redelegate(.mock(), validators: [.mock()], recommended: nil)).title == "Redelegate")
        #expect(AmountStakeViewModel(asset: .mockBNB(), action: .withdraw(.mock())).title == "Withdraw")
    }

    @Test
    func validatorSelectionEnabled() {
        #expect(AmountStakeViewModel(asset: .mockBNB(), action: .stake(validators: [.mock()], recommended: nil)).validatorSelection.isEnabled == true)
        #expect(AmountStakeViewModel(asset: .mockBNB(), action: .unstake(.mock())).validatorSelection.isEnabled == false)
        #expect(AmountStakeViewModel(asset: .mockBNB(), action: .redelegate(.mock(), validators: [.mock()], recommended: nil)).validatorSelection.isEnabled == true)
        #expect(AmountStakeViewModel(asset: .mockBNB(), action: .withdraw(.mock())).validatorSelection.isEnabled == false)
    }

    @Test
    func validatorSelection() {
        let recommended = DelegationValidator.mock(id: "recommended")
        let first = DelegationValidator.mock(id: "first")

        let withRecommended = AmountStakeViewModel(asset: .mockBNB(), action: .stake(validators: [first, recommended], recommended: recommended))
        let withoutRecommended = AmountStakeViewModel(asset: .mockBNB(), action: .stake(validators: [first, recommended], recommended: nil))

        #expect(withRecommended.validatorSelection.selected.id == "recommended")
        #expect(withoutRecommended.validatorSelection.selected.id == "first")
    }

    @Test
    func validatorSelectType() {
        #expect(AmountStakeViewModel(asset: .mockBNB(), action: .stake(validators: [.mock()], recommended: nil)).validatorSelectType == .stake)
        #expect(AmountStakeViewModel(asset: .mockBNB(), action: .redelegate(.mock(), validators: [.mock()], recommended: nil)).validatorSelectType == .stake)
        #expect(AmountStakeViewModel(asset: .mockBNB(), action: .unstake(.mock())).validatorSelectType == .unstake)
        #expect(AmountStakeViewModel(asset: .mockBNB(), action: .withdraw(.mock())).validatorSelectType == .unstake)
    }

    @Test
    func canChangeValue() {
        #expect(AmountStakeViewModel(asset: .mockBNB(), action: .stake(validators: [.mock()], recommended: nil)).canChangeValue == true)
        #expect(AmountStakeViewModel(asset: .mockBNB(), action: .redelegate(.mock(), validators: [.mock()], recommended: nil)).canChangeValue == true)
        #expect(AmountStakeViewModel(asset: .mockBNB(), action: .withdraw(.mock())).canChangeValue == false)
    }

    @Test
    func reserveForFee() {
        #expect(AmountStakeViewModel(asset: .mockBNB(), action: .stake(validators: [.mock()], recommended: nil)).reserveForFee > .zero)
        #expect(AmountStakeViewModel(asset: .mockTron(), action: .stake(validators: [.mock()], recommended: nil)).reserveForFee == .zero)
        #expect(AmountStakeViewModel(asset: .mockBNB(), action: .unstake(.mock())).reserveForFee == .zero)
    }

    @Test
    func minimumValue() {
        #expect(AmountStakeViewModel(asset: .mockBNB(), action: .stake(validators: [.mock()], recommended: nil)).minimumValue > .zero)
        #expect(AmountStakeViewModel(asset: .mockBNB(), action: .unstake(.mock())).minimumValue == .zero)
    }

    @Test
    func availableValue() {
        let delegation = Delegation.mock(base: .mock(state: .active, balance: "5000000"))
        let assetData = AssetData.mock(asset: .mockBNB(), balance: .mock(available: 1000))

        let stake = AmountStakeViewModel(asset: .mockBNB(), action: .stake(validators: [.mock()], recommended: nil))
        let unstake = AmountStakeViewModel(asset: .mockBNB(), action: .unstake(delegation))

        #expect(stake.availableValue(from: assetData) == 1000)
        #expect(unstake.availableValue(from: assetData) == 5000000)
    }

    @Test
    func shouldReserveFee() {
        let assetData = AssetData.mock(asset: .mockBNB(), balance: .mock(available: 5_000_000_000_000_000_000))
        let delegation = Delegation.mock(base: .mock(state: .active, balance: "1000000"))
        let unstake = AmountStakeViewModel(asset: .mockBNB(), action: .unstake(delegation))

        #expect(unstake.shouldReserveFee(from: assetData) == false)
    }

    @Test
    func makeTransferData() throws {
        let validator = DelegationValidator.mock(id: "validator1")
        let delegation = Delegation.mock(validator: validator)

        let stake = try AmountStakeViewModel(asset: .mockBNB(), action: .stake(validators: [validator], recommended: nil)).makeTransferData(value: 100)
        let unstake = try AmountStakeViewModel(asset: .mockBNB(), action: .unstake(delegation)).makeTransferData(value: 100)
        let redelegate = try AmountStakeViewModel(asset: .mockBNB(), action: .redelegate(delegation, validators: [validator], recommended: nil)).makeTransferData(value: 100)
        let withdraw = try AmountStakeViewModel(asset: .mockBNB(), action: .withdraw(delegation)).makeTransferData(value: 100)

        #expect(stake.type.transactionType == .stakeDelegate)
        #expect(unstake.type.transactionType == .stakeUndelegate)
        #expect(redelegate.type.transactionType == .stakeRedelegate)
        #expect(withdraw.type.transactionType == .stakeWithdraw)
        #expect(stake.value == 100)
        #expect(unstake.value == 100)
        #expect(redelegate.value == 100)
        #expect(withdraw.value == 100)
    }
}
