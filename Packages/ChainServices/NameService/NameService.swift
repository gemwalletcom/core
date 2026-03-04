// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GemAPI
import Primitives

public final class NameService: NameServiceable, Sendable {
    private let provider: any GemAPINameService

    public init(
        provider: any GemAPINameService = GemAPIService()
    ) {
        self.provider = provider
    }
    
    public func getName(name: String, chain: String) async throws -> NameRecord? {
        try await provider.getName(name: name, chain: chain)
    }
}
