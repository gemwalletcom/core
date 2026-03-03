// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import Primitives

enum AmountDataProvider: AmountDataProvidable, @unchecked Sendable {
    case transfer(AmountTransferViewModel)
    case stake(AmountStakeViewModel)
    case freeze(AmountFreezeViewModel)
    case perpetual(AmountPerpetualViewModel)
    case earn(AmountEarnViewModel)

    static func make(
        from input: AmountInput,
        wallet: Wallet,
        service: AmountService
    ) -> AmountDataProvider {
        switch input.type {
        case .transfer(let recipient):
            .transfer(AmountTransferViewModel(asset: input.asset, action: .send(recipient)))
        case .deposit(let recipient):
            .transfer(AmountTransferViewModel(asset: input.asset, action: .deposit(recipient)))
        case .withdraw(let recipient):
            .transfer(AmountTransferViewModel(asset: input.asset, action: .withdraw(recipient)))
        case .stake(let stakeType):
            .stake(AmountStakeViewModel(asset: input.asset, action: stakeType))
        case .freeze(let data):
            .freeze(AmountFreezeViewModel(asset: input.asset, data: data))
        case .perpetual(let data):
            .perpetual(AmountPerpetualViewModel(asset: input.asset, data: data))
        case .earn(let earnType):
            .earn(AmountEarnViewModel(asset: input.asset, action: earnType, earnService: service.earnDataProvider, wallet: wallet))
        }
    }

    var asset: Asset { provider.asset }
    var title: String { provider.title }
    var amountType: AmountType { provider.amountType }
    var minimumValue: BigInt { provider.minimumValue }
    var canChangeValue: Bool { provider.canChangeValue }
    var reserveForFee: BigInt { provider.reserveForFee }

    func availableValue(from assetData: AssetData) -> BigInt {
        provider.availableValue(from: assetData)
    }

    func shouldReserveFee(from assetData: AssetData) -> Bool {
        provider.shouldReserveFee(from: assetData)
    }

    func maxValue(from assetData: AssetData) -> BigInt {
        provider.maxValue(from: assetData)
    }

    func recipientData() -> RecipientData {
        provider.recipientData()
    }

    func makeTransferData(value: BigInt) async throws -> TransferData {
        try await provider.makeTransferData(value: value)
    }
}

// MARK: - Private

extension AmountDataProvider {
    private var provider: any AmountDataProvidable {
        switch self {
        case .transfer(let provider): provider
        case .stake(let provider): provider
        case .freeze(let provider): provider
        case .perpetual(let provider): provider
        case .earn(let provider): provider
        }
    }
}
