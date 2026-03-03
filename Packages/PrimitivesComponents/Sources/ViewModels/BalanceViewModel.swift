// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Formatters
import Foundation
import Primitives
import Style
import SwiftUI

public struct BalanceViewModel: Sendable {
    private static let fullFormatter = ValueFormatter(style: .full)

    private let asset: Asset
    private let balance: Balance
    private let formatter: ValueFormatter

    public init(
        asset: Asset,
        balance: Balance,
        formatter: ValueFormatter
    ) {
        self.asset = asset
        self.balance = balance
        self.formatter = formatter
    }

    public var balanceAmount: Double {
        do {
            return try Self.fullFormatter.double(from: total, decimals: asset.decimals.asInt)
        } catch {
            return .zero
        }
    }

    public var availableBalanceAmount: Double {
        do {
            return try Self.fullFormatter.double(from: balance.available, decimals: asset.decimals.asInt)
        } catch {
            return .zero
        }
    }

    public var balanceText: String {
        guard !total.isZero else {
            return .zero
        }
        return formatter.string(total, decimals: asset.decimals.asInt)
    }

    public var availableBalanceText: String {
        guard !balance.available.isZero else {
            return .zero
        }
        return formatter.string(balance.available, decimals: asset.decimals.asInt)
    }

    public var totalBalanceTextWithSymbol: String {
        formatter.string(total, decimals: asset.decimals.asInt, currency: asset.symbol)
    }

    public var availableBalanceTextWithSymbol: String {
        formatter.string(balance.available, decimals: asset.decimals.asInt, currency: asset.symbol)
    }

    public func balanceTextWithSymbol(for type: StakeProviderType) -> String {
        let amount = switch type {
        case .stake:
            switch StakeChain(rawValue: asset.chain.rawValue) {
            case .celestia, .cosmos, .hyperCore, .injective, .osmosis, .sei, .smartChain, .solana, .sui, .ethereum, .aptos, .monad, .none:
                balance.staked + balance.pending
            case .tron:
                balance.frozen + balance.locked + balance.pending
            }
        case .earn:
            balance.earn
        }
        return formatter.string(amount, decimals: asset.decimals.asInt, currency: asset.symbol)
    }

    public var hasStakingResources: Bool {
        switch StakeChain(rawValue: asset.chain.rawValue) {
        case .celestia, .cosmos, .hyperCore, .injective, .osmosis, .sei, .smartChain, .solana, .sui, .ethereum, .aptos, .monad, .none:
            false
        case .tron:
            true
        }
    }

    public var hasReservedBalance: Bool {
        !balance.reserved.isZero
    }

    public var reservedBalanceTextWithSymbol: String {
        formatter.string(balance.reserved, decimals: asset.decimals.asInt, currency: asset.symbol)
    }

    public var hasPendingUnconfirmedBalance: Bool {
        !balance.pendingUnconfirmed.isZero
    }

    public var pendingUnconfirmedBalanceTextWithSymbol: String {
        formatter.string(balance.pendingUnconfirmed, decimals: asset.decimals.asInt, currency: asset.symbol)
    }

    public var balanceTextColor: Color {
        guard !total.isZero else {
            return Colors.gray
        }
        return Colors.black
    }

    public var energyText: String {
        guard let metadata = balance.metadata else { return "" }
        return "\(metadata.energyAvailable) / \(metadata.energyTotal)"
    }

    public var bandwidthText: String {
        guard let metadata = balance.metadata else { return "" }
        return "\(metadata.bandwidthAvailable) / \(metadata.bandwidthTotal)"
    }

    var total: BigInt {
        balance.available + balance.frozen + balance.locked + balance.staked + balance.pending + balance.rewards + balance.earn
    }
}
