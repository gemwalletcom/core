// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

public final class NodeAuthTokenProvider: RequestInterceptable {
    private let securePreferences: SecurePreferences

    public init(securePreferences: SecurePreferences) {
        self.securePreferences = securePreferences
    }

    public func intercept(request: inout URLRequest) {
        guard let host = request.url?.host,
              let nodesDomain = Constants.nodesURL.host,
              host == nodesDomain || host.hasSuffix(".\(nodesDomain)") else { return }
        guard let token = try? securePreferences.nodeAuthToken(),
              token.expiresAt > UInt64(Date.now.timeIntervalSince1970) else { return }
        request.setValue("Bearer \(token.token)", forHTTPHeaderField: "Authorization")
    }
}
