// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import BigInt

public enum SimulationWarningType: Codable, Equatable, Hashable, Sendable {
    case tokenApproval(assetId: AssetId, value: BigInt?)
    case suspiciousSpender
    case externallyOwnedSpender
    case nftCollectionApproval(assetId: AssetId)
    case permitApproval(assetId: AssetId, value: BigInt?)
    case permitBatchApproval(value: BigInt?)
    case validationError
}
