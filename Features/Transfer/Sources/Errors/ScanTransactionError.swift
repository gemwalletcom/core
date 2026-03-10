// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import Localization

enum ScanTransactionError: Error, Equatable, Sendable {
    case malicious
    case memoRequired(symbol: String)
}

extension ScanTransactionError: LocalizedError {
    var errorDescription: String? {
        switch self {
        case .malicious: Localized.Errors.ScanTransaction.Malicious.description
        case .memoRequired(let symbol): Localized.Errors.ScanTransaction.memoRequired(symbol.boldMarkdown())
        }
    }
}
