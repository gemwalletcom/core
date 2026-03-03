// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public enum EmptyContentType {
    public enum SearchType {
        case assets
        case networks
        case activity
    }

    case nfts(action: (() -> Void)? = nil)
    case priceAlerts
    case asset(symbol: String, buy: (() -> Void)? = nil, swap: (() -> Void)? = nil, isViewOnly: Bool)
    case activity(receive: (() -> Void)? = nil, buy: (() -> Void)? = nil, isViewOnly: Bool)
    case stake(symbol: String)
    case earn(symbol: String)
    case walletConnect
    case search(type: SearchType, action: (() -> Void)? = nil)
    case markets
    case recents
    case notifications
    case contacts
}
