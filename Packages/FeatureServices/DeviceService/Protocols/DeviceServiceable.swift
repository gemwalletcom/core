// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public protocol DeviceServiceable: Sendable {
    func prepareForWalletRequest() async throws
    func update() async throws
    func updateNodeAuthTokenIfNeeded() async throws
}
