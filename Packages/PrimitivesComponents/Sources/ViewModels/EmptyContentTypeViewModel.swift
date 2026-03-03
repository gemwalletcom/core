// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import SwiftUI
import Style
import Components
import Localization

public struct EmptyContentTypeViewModel: EmptyContentViewable {
    public let type: EmptyContentType

    public init(type: EmptyContentType) {
        self.type = type
    }

    public var title: String {
        switch type {
        case .nfts: Localized.Nft.State.Empty.title
        case .priceAlerts: Localized.PriceAlerts.State.Empty.title
        case let .asset(_, _, _, isViewOnly):
            switch isViewOnly {
            case true: Localized.Wallet.Watch.Tooltip.title
            case false: Localized.Asset.State.Empty.title
            }
        case let .activity(_, _, isViewOnly):
            switch isViewOnly {
            case true: Localized.Wallet.Watch.Tooltip.title
            case false: Localized.Activity.State.Empty.title
            }
        case .stake: Localized.Stake.State.Empty.title
        case .earn: Localized.Earn.State.Empty.title
        case .walletConnect: Localized.WalletConnect.noActiveConnections
        case .markets: Localized.Markets.State.Empty.title
        case .notifications: Localized.Notifications.Inapp.State.Empty.title
        case let .search(searchType, _):
            switch searchType {
            case .assets: Localized.Assets.noAssetsFound
            case .networks: Localized.Networks.State.Empty.searchTitle
            case .activity: Localized.Activity.State.Empty.searchTitle
            }
        case .recents: Localized.RecentActivity.State.Empty.title
        case .contacts: Localized.Contacts.State.Empty.title
        }
    }

    public var description: String? {
        switch type {
        case let .nfts(action): action != nil ? Localized.Nft.State.Empty.description : nil
        case .priceAlerts: Localized.PriceAlerts.State.Empty.description
        case let .asset(symbol, _, _, isViewOnly):
            switch isViewOnly {
            case true: Localized.Info.WatchWallet.description
            case false: Localized.Asset.State.Empty.description(symbol)
            }
        case let .activity(_, _, isViewOnly):
            switch isViewOnly {
            case true: Localized.Info.WatchWallet.description
            case false: Localized.Activity.State.Empty.description
            }
        case let .stake(symbol): Localized.Stake.State.Empty.description(symbol)
        case let .earn(symbol): Localized.Earn.State.Empty.description(symbol)
        case .walletConnect: Localized.WalletConnect.State.Empty.description
        case let .search(searchType, action):
            switch searchType {
            case .assets: action != nil ? Localized.Assets.State.Empty.searchDescription : Localized.Search.State.Empty.description
            case .networks: Localized.Search.State.Empty.description
            case .activity: Localized.Activity.State.Empty.searchDescription
            }
        case .markets: .none
        case .recents: Localized.RecentActivity.State.Empty.description
        case .notifications: Localized.Notifications.Inapp.State.Empty.description
        case .contacts: Localized.Contacts.State.Empty.description
        }
    }

    public var image: Image? {
        switch type {
        case .nfts: Images.EmptyContent.nft
        case .priceAlerts: Images.EmptyContent.priceAlerts
        case .asset, .activity: Images.EmptyContent.activity
        case .stake: Images.EmptyContent.stake
        case .earn: Images.EmptyContent.stake
        case .walletConnect: Images.EmptyContent.walletConnect
        case .search: Images.EmptyContent.search
        case .markets, .recents: Images.EmptyContent.activity
        case .notifications: Images.System.bell
        case .contacts: Images.EmptyContent.contacts
        }
    }

    public var buttons: [EmptyAction] {
        let actions: [EmptyAction]

        switch type {
        case .priceAlerts, .stake, .earn, .walletConnect, .markets, .notifications, .recents, .contacts:
            actions = []
        case let .asset(_, buy, swap, isViewOnly):
            switch isViewOnly {
            case true: actions = []
            case false:
                if let buy {
                    actions = [EmptyAction(title: Localized.Wallet.buy, action: buy)]
                } else if let swap {
                    actions = [EmptyAction(title: Localized.Wallet.swap, action: swap)]
                } else {
                    actions = []
                }
            }
        case let .nfts(action):
            actions = [EmptyAction(title: Localized.Wallet.receive, action: action)]
        case let .activity(receive, buy, isViewOnly):
            switch isViewOnly {
            case true: actions = []
            case false : actions = [
                    EmptyAction(title: Localized.Wallet.buy, action: buy),
                    EmptyAction(title: Localized.Wallet.receive, action: receive)
                ]
            }
        case let .search(searchType, action):
            switch searchType {
            case .assets:
                actions = [EmptyAction(title: Localized.Assets.addCustomToken, action: action)]
            case .networks:
                actions = []
            case .activity:
                actions = [EmptyAction(title: Localized.Filter.clear, action: action)]
            }
        }

        return actions.filter { $0.action != nil }
    }
}
