// Copyright (c). Gem Wallet. All rights reserved.

import Testing
import Localization
@testable import Transfer
@testable import Primitives
import PrimitivesTestKit
import TransferTestKit

struct ConfirmRecipientViewModelTests {

    @Test
    func transfer() {
        let model = ConfirmRecipientViewModel(model: .mock(type: .transfer(.mock())), addressName: nil, addressLink: .mock())

        guard case .recipient(let item) = model.itemModel else { return }
        #expect(item.title == Localized.Transfer.Recipient.title)
        #expect(item.account.address != "")
    }

    @Test
    func transferNft() {
        let model = ConfirmRecipientViewModel(model: .mock(type: .transferNft(.mock())), addressName: nil, addressLink: .mock())

        guard case .recipient(let item) = model.itemModel else { return }
        #expect(item.title == Localized.Transfer.Recipient.title)
    }

    @Test
    func deposit() {
        let model = ConfirmRecipientViewModel(model: .mock(type: .deposit(.mock())), addressName: nil, addressLink: .mock())

        guard case .recipient(let item) = model.itemModel else { return }
        #expect(item.title == Localized.Transfer.Recipient.title)
    }

    @Test
    func withdrawal() {
        let model = ConfirmRecipientViewModel(model: .mock(type: .withdrawal(.mock())), addressName: nil, addressLink: .mock())

        guard case .recipient(let item) = model.itemModel else { return }
        #expect(item.title == Localized.Transfer.Recipient.title)
    }

    @Test
    func tokenApprove() {
        let model = ConfirmRecipientViewModel(model: .mock(type: .tokenApprove(.mock(), .mock())), addressName: nil, addressLink: .mock())

        guard case .recipient(let item) = model.itemModel else { return }
        #expect(item.title == Localized.Transfer.Recipient.title)
    }

    @Test
    func genericSend() {
        let model = ConfirmRecipientViewModel(
            model: .mock(type: .generic(asset: .mock(), metadata: .mock(), extra: .mock(outputAction: .send))),
            addressName: nil,
            addressLink: .mock()
        )

        guard case .recipient(let item) = model.itemModel else { return }
        #expect(item.title == Localized.Transfer.Recipient.title)
    }

    @Test
    func genericSign() {
        let model = ConfirmRecipientViewModel(
            model: .mock(type: .generic(asset: .mock(), metadata: .mock(), extra: .mock(outputAction: .sign))),
            addressName: nil,
            addressLink: .mock()
        )

        guard case .recipient(let item) = model.itemModel else { return }
        #expect(item.title == Localized.Asset.contract)
    }

    @Test
    func stakeDelegate() {
        let model = ConfirmRecipientViewModel(model: .mock(type: .stake(.mock(), .stake(.mock()))), addressName: nil, addressLink: .mock())

        guard case .recipient(let item) = model.itemModel else { return }
        #expect(item.title == Localized.Stake.validator)
    }

    @Test
    func stakeUndelegate() {
        let model = ConfirmRecipientViewModel(model: .mock(type: .stake(.mock(), .unstake(.mock()))), addressName: nil, addressLink: .mock())

        guard case .recipient(let item) = model.itemModel else { return }
        #expect(item.title == Localized.Stake.validator)
    }

    @Test
    func stakeRedelegate() {
        let model = ConfirmRecipientViewModel(
            model: .mock(type: .stake(.mock(), .redelegate(RedelegateData(delegation: .mock(), toValidator: .mock())))),
            addressName: nil,
            addressLink: .mock()
        )

        guard case .recipient(let item) = model.itemModel else { return }
        #expect(item.title == Localized.Stake.validator)
    }

    @Test
    func stakeWithdraw() {
        let model = ConfirmRecipientViewModel(model: .mock(type: .stake(.mock(), .withdraw(.mock()))), addressName: nil, addressLink: .mock())

        guard case .recipient(let item) = model.itemModel else { return }
        #expect(item.title == Localized.Stake.validator)
    }

    @Test
    func stakeFreeze() {
        let model = ConfirmRecipientViewModel(
            model: .mock(type: .stake(.mock(), .freeze(.bandwidth))),
            addressName: nil,
            addressLink: .mock()
        )

        guard case .recipient(let item) = model.itemModel else { return }
        #expect(item.title == Localized.Stake.resource)
    }

    @Test
    func addressName() {
        let addressName = AddressName.mock(name: "Vitalik.eth")
        let model = ConfirmRecipientViewModel(model: .mock(type: .transfer(.mock())), addressName: addressName, addressLink: .mock())

        guard case .recipient(let item) = model.itemModel else { return }
        #expect(item.account.name == "Vitalik.eth")
    }
}

private extension TransferDataViewModel {
    static func mock(type: TransferDataType = .transfer(.mock())) -> TransferDataViewModel {
        TransferDataViewModel(data: .mock(type: type))
    }
}
