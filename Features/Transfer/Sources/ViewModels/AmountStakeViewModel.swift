// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Formatters
import Foundation
import Gemstone
import GemstonePrimitives
import Localization
import Primitives
import PrimitivesComponents
import Stake
import Validators

final class AmountStakeViewModel: AmountDataProvidable {
    let asset: Asset
    let action: StakeAmountType
    let validatorSelection: SelectionState<DelegationValidator>

    init(asset: Asset, action: StakeAmountType) {
        self.asset = asset
        self.action = action
        self.validatorSelection = Self.makeValidatorSelection(action: action)
    }

    private static func makeValidatorSelection(action: StakeAmountType) -> SelectionState<DelegationValidator> {
        switch action {
        case let .stake(validators, recommended):
            SelectionState(options: validators, selected: recommended ?? validators[0], isEnabled: true, title: Localized.Stake.validator)
        case let .unstake(delegation):
            SelectionState(options: [delegation.validator], selected: delegation.validator, isEnabled: false, title: Localized.Stake.validator)
        case let .redelegate(_, validators, recommended):
            SelectionState(options: validators, selected: recommended ?? validators[0], isEnabled: true, title: Localized.Stake.validator)
        case let .withdraw(delegation):
            SelectionState(options: [delegation.validator], selected: delegation.validator, isEnabled: false, title: Localized.Stake.validator)
        }
    }

    var validatorSelectType: ValidatorSelectType {
        switch action {
        case .stake, .redelegate: .stake
        case .unstake, .withdraw: .unstake
        }
    }

    var title: String {
        switch action {
        case .stake: Localized.Transfer.Stake.title
        case .unstake: Localized.Transfer.Unstake.title
        case .redelegate: Localized.Transfer.Redelegate.title
        case .withdraw: Localized.Transfer.Withdraw.title
        }
    }

    var amountType: AmountType {
        .stake(action)
    }

    var minimumValue: BigInt {
        guard let stakeChain = asset.chain.stakeChain else { return .zero }
        return switch action {
        case .stake:
            BigInt(StakeConfig.config(chain: stakeChain).minAmount)
        case .redelegate:
            stakeChain == .smartChain ? BigInt(StakeConfig.config(chain: stakeChain).minAmount) : .zero
        case .unstake:
            .zero
        case .withdraw:
            asset.symbol == "USDC" ? AmountPerpetualLimits.minDeposit : .zero
        }
    }

    var canChangeValue: Bool {
        switch action {
        case .stake, .redelegate:
            true
        case .unstake:
            StakeChain(rawValue: asset.chain.rawValue)?.canChangeAmountOnUnstake ?? true
        case .withdraw:
            false
        }
    }

    func shouldReserveFee(from assetData: AssetData) -> Bool {
        let maxAfterFee = max(.zero, availableValue(from: assetData) - reserveForFee)
        return switch action {
        case .stake:
            asset.chain != .tron && maxAfterFee > minimumValue && !reserveForFee.isZero
        case .unstake, .redelegate, .withdraw:
            false
        }
    }

    var reserveForFee: BigInt {
        switch action {
        case .stake where asset.chain != .tron:
            BigInt(Gemstone.Config.shared.getStakeConfig(chain: asset.chain.rawValue).reservedForFees)
        default:
            .zero
        }
    }

    func availableValue(from assetData: AssetData) -> BigInt {
        switch action {
        case .stake:
            if asset.chain == .tron {
                let staked = BigNumberFormatter.standard.number(
                    from: Int(assetData.balance.metadata?.votes ?? 0),
                    decimals: Int(assetData.asset.decimals)
                )
                return (assetData.balance.frozen + assetData.balance.locked) - staked
            }
            return assetData.balance.available
        case .unstake(let delegation), .redelegate(let delegation, _, _), .withdraw(let delegation):
            return delegation.base.balanceValue
        }
    }

    func recipientData() -> RecipientData {
        RecipientData(
            recipient: Recipient(
                name: validatorSelection.selected.name,
                address: recipientAddress(validatorId: validatorSelection.selected.id),
                memo: Localized.Stake.viagem
            ),
            amount: nil
        )
    }

    func makeTransferData(value: BigInt) throws -> TransferData {
        let stakeType: StakeType = switch action {
        case .stake:
            .stake(validatorSelection.selected)
        case .unstake(let delegation):
            .unstake(delegation)
        case .redelegate(let delegation, _, _):
            .redelegate(RedelegateData(delegation: delegation, toValidator: validatorSelection.selected))
        case .withdraw(let delegation):
            .withdraw(delegation)
        }
        return TransferData(
            type: .stake(asset, stakeType),
            recipientData: recipientData(),
            value: value,
            canChangeValue: canChangeValue
        )
    }

    private func recipientAddress(validatorId: String) -> String {
        switch asset.chain.stakeChain {
        case .cosmos, .osmosis, .injective, .sei, .celestia, .solana, .sui, .tron, .smartChain, .ethereum, .aptos, .monad:
            validatorId
        case .none, .hyperCore:
            ""
        }
    }
}
