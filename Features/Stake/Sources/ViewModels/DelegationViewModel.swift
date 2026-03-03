// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import Components
import PrimitivesComponents
import SwiftUI
import Style
import Formatters
import ExplorerService

public struct DelegationViewModel: Sendable {

    public let delegation: Delegation
    public let currencyCode: String
    private let asset: Asset
    private let formatter: ValueFormatter
    private let exploreService: ExplorerService
    private let priceFormatter: CurrencyFormatter

    private static let dateFormatterDefault: DateComponentsFormatter = {
        let formatter = DateComponentsFormatter()
        formatter.allowedUnits = [.day, .hour]
        formatter.zeroFormattingBehavior = .dropLeading
        formatter.unitsStyle = .full
        return formatter
    }()

    private static let dateFormatterDay: DateComponentsFormatter = {
        let formatter = DateComponentsFormatter()
        formatter.allowedUnits = [.hour, .minute]
        formatter.zeroFormattingBehavior = .dropLeading
        formatter.unitsStyle = .full
        return formatter
    }()

    public init(
        delegation: Delegation,
        asset: Asset,
        formatter: ValueFormatter = .short,
        currencyCode: String,
        exploreService: ExplorerService = .standard
    ) {
        self.delegation = delegation
        self.currencyCode = currencyCode
        self.asset = asset
        self.formatter = formatter
        self.exploreService = exploreService
        self.priceFormatter = CurrencyFormatter(type: .currency, currencyCode: currencyCode)
    }

    public var state: DelegationState {
        delegation.base.state
    }

    public var stateModel: DelegationStateViewModel {
        DelegationStateViewModel(state: state)
    }

    public var titleStyle: TextStyle {
        TextStyle(font: .body, color: .primary, fontWeight: .semibold)
    }

    public var subtitleStyle: TextStyle {
        TextStyle(font: .callout, color: Colors.black, fontWeight: .semibold)
    }

    public var subtitleExtraStyle: TextStyle {
        TextStyle(font: .footnote, color: Colors.gray)
    }

    public var balanceText: String {
        formatter.string(delegation.base.balanceValue, decimals: asset.decimals.asInt, currency: asset.symbol)
    }

    public var fiatValueText: String? {
        guard
            let price = delegation.price,
            let balance = try? formatter.double(from: delegation.base.balanceValue, decimals: asset.decimals.asInt)
        else { return nil }
        return priceFormatter.string(price.price * balance)
    }

    public var rewardsText: String? {
        switch delegation.base.state {
        case .active:
            if delegation.base.rewardsValue == 0 {
                return .none
            }
            return formatter.string(delegation.base.rewardsValue, decimals: asset.decimals.asInt, currency: asset.symbol)
        case .pending,
            .inactive,
            .activating,
            .deactivating,
            .awaitingWithdrawal:
            return .none
        }
    }

    public var rewardsFiatValueText: String? {
        guard
            let price = delegation.price,
            delegation.base.rewardsValue > 0,
            let rewards = try? formatter.double(from: delegation.base.rewardsValue, decimals: asset.decimals.asInt)
        else { return nil }
        return priceFormatter.string(price.price * rewards)
    }

    public var validatorModel: ValidatorViewModel {
        ValidatorViewModel(validator: delegation.validator, exploreService: exploreService)
    }

    public var validatorText: String { validatorModel.name }
    public var validatorImage: AssetImage { validatorModel.validatorImage }
    public var validatorUrl: URL? { validatorModel.url }

    public var completionDateText: String? {
        let now = Date.now
        if let completionDate = delegation.base.completionDate, completionDate > now {
            if now.distance(to: completionDate) < 86400 {
                return Self.dateFormatterDay.string(from: .now, to: completionDate)
            }
            return Self.dateFormatterDefault.string(from: .now, to: completionDate)
        }
        return .none
    }
}

extension DelegationViewModel: Identifiable {
    public var id: String { delegation.id }
}

// MARK: - HeaderViewModel

extension DelegationViewModel: HeaderViewModel {
    public var isWatchWallet: Bool { false }
    public var buttons: [HeaderButton] { [] }
    public var assetImage: AssetImage? { validatorImage }
    public var title: String { balanceText }
    public var subtitle: String? { fiatValueText }
    public var subtitleColor: Color { .secondary }
}
