// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI
import Components

public protocol HeaderViewModel {
    var isWatchWallet: Bool { get }
    var assetImage: AssetImage? { get }
    var title: String { get }
    var subtitle: String? { get }
    var subtitleImage: Image? { get }
    var subtitleColor: Color { get }
    var buttons: [HeaderButton] { get }
}

extension HeaderViewModel {
    public var subtitleImage: Image? { nil }
}
