// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

public protocol AddAssetServiceable: Sendable {
    func getTokenData(chain: Chain, tokenId: String) async throws -> Asset
}
