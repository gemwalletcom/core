// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import Gemstone
import GemstonePrimitives
import Localization
import Primitives
import PrimitivesComponents
import Stake

final class AmountFreezeViewModel: AmountDataProvidable {
    let asset: Asset
    let data: FreezeData
    let resourceSelection: SelectionState<Resource>

    init(asset: Asset, data: FreezeData) {
        self.asset = asset
        self.data = data
        self.resourceSelection = SelectionState(
            options: [.bandwidth, .energy],
            selected: data.resource,
            isEnabled: true,
            title: Localized.Stake.resource
        )
    }

    var title: String {
        data.freezeType == .freeze ? Localized.Transfer.Freeze.title : Localized.Transfer.Unfreeze.title
    }

    var amountType: AmountType {
        .freeze(data: data)
    }

    var minimumValue: BigInt {
        guard let stakeChain = asset.chain.stakeChain else { return .zero }
        return data.freezeType == .freeze ? BigInt(StakeConfig.config(chain: stakeChain).minAmount) : .zero
    }

    var canChangeValue: Bool { true }

    var reserveForFee: BigInt {
        guard data.freezeType == .freeze else { return .zero }
        return BigInt(Gemstone.Config.shared.getStakeConfig(chain: asset.chain.rawValue).reservedForFees)
    }

    func shouldReserveFee(from assetData: AssetData) -> Bool {
        let maxAfterFee = max(.zero, availableValue(from: assetData) - reserveForFee)
        return data.freezeType == .freeze && maxAfterFee > minimumValue
    }

    func availableValue(from assetData: AssetData) -> BigInt {
        switch data.freezeType {
        case .freeze: assetData.balance.available
        case .unfreeze: resourceSelection.selected == .bandwidth ? assetData.balance.frozen : assetData.balance.locked
        }
    }

    func recipientData() -> RecipientData {
        let resource = resourceSelection.selected
        let title = ResourceViewModel(resource: resource).title
        return RecipientData(
            recipient: Recipient(name: title, address: title, memo: nil),
            amount: nil
        )
    }

    func makeTransferData(value: BigInt) throws -> TransferData {
        let stakeType: StakeType = .freeze(FreezeData(freezeType: data.freezeType, resource: resourceSelection.selected))
        return TransferData(
            type: .stake(asset, stakeType),
            recipientData: recipientData(),
            value: value,
            canChangeValue: canChangeValue
        )
    }
}
