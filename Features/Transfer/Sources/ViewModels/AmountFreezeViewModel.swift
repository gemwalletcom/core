// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Gemstone
import GemstonePrimitives
import Localization
import Primitives
import PrimitivesComponents
import Stake

final class AmountFreezeViewModel: AmountDataProvidable {
    enum Action {
        case freeze
        case unfreeze
    }

    let asset: Asset
    let action: Action
    let resourceSelection: SelectionState<Resource>

    init(asset: Asset, action: Action, resource: Resource) {
        self.asset = asset
        self.action = action
        self.resourceSelection = SelectionState(
            options: [.bandwidth, .energy],
            selected: resource,
            isEnabled: true,
            title: Localized.Stake.resource
        )
    }

    var title: String {
        switch action {
        case .freeze: Localized.Transfer.Freeze.title
        case .unfreeze: Localized.Transfer.Unfreeze.title
        }
    }

    var amountType: AmountType {
        switch action {
        case .freeze: .freeze(resource: resourceSelection.selected)
        case .unfreeze: .unfreeze(resource: resourceSelection.selected)
        }
    }

    var minimumValue: BigInt {
        guard let stakeChain = asset.chain.stakeChain else { return .zero }
        return switch action {
        case .freeze: BigInt(StakeConfig.config(chain: stakeChain).minAmount)
        case .unfreeze: BigInt.zero
        }
    }

    var canChangeValue: Bool { true }

    var reserveForFee: BigInt {
        return switch action {
        case .freeze: BigInt(Gemstone.Config.shared.getStakeConfig(chain: asset.chain.rawValue).reservedForFees)
        case .unfreeze: BigInt.zero
        }
    }

    func shouldReserveFee(from assetData: AssetData) -> Bool {
        let maxAfterFee = max(.zero, availableValue(from: assetData) - reserveForFee)
        return switch action {
        case .freeze: maxAfterFee > minimumValue
        case .unfreeze: false
        }
    }

    func availableValue(from assetData: AssetData) -> BigInt {
        return switch action {
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
        let stakeType: StakeType = switch action {
        case .freeze: .freeze(resourceSelection.selected)
        case .unfreeze: .unfreeze(resourceSelection.selected)
        }
        return TransferData(
            type: .stake(asset, stakeType),
            recipientData: recipientData(),
            value: value,
            canChangeValue: canChangeValue
        )
    }
}
