// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Components

public struct GridPosterViewItem: Identifiable, Sendable {
    public let id: String
    public let destination: any Hashable & Sendable
    public let model: GridPosterViewModel

    public init(
        id: String,
        destination: any Hashable & Sendable,
        model: GridPosterViewModel
    ) {
        self.id = id
        self.destination = destination
        self.model = model
    }
}
