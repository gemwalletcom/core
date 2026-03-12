// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public extension AssetScoreType {
    init(verificationStatus: VerificationStatus) {
        switch verificationStatus {
        case .verified: self = .verified
        case .unverified: self = .unverified
        case .suspicious: self = .suspicious
        }
    }
}
