// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public struct GridPosterViewModel: Sendable {
    public let assetImage: AssetImage
    public let title: String?
    public let count: Int?
    public let isVerified: Bool

    public init(
        assetImage: AssetImage,
        title: String?,
        count: Int? = nil,
        isVerified: Bool = false
    ) {
        self.assetImage = assetImage
        self.title = title
        self.count = count
        self.isVerified = isVerified
    }
}
