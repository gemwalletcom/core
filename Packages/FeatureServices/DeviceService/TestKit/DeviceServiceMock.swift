// Copyright (c). Gem Wallet. All rights reserved.

import DeviceService

public actor DeviceServiceMock: DeviceServiceable {
    public private(set) var updateCalls = 0

    public init() {}

    public func update() async throws {
        updateCalls += 1
    }

    public func updateNodeAuthTokenIfNeeded() async throws {
    }

    public func prepareForWalletRequest() async throws {
    }
}
