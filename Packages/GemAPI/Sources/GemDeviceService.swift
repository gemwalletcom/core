// Copyright (c). Gem Wallet. All rights reserved.

import Primitives
import SwiftHTTPClient

public struct GemDeviceService {
    private let deviceProvider: Provider<GemDeviceAPI>

    public init(deviceProvider: Provider<GemDeviceAPI>) {
        self.deviceProvider = deviceProvider
    }

    private func request(_ target: GemDeviceAPI) async throws -> Response {
        try await deviceProvider.request(target)
    }
}

extension GemDeviceService: GemAPIDeviceService {
    public func getDevice() async throws -> Device? {
        try await request(.getDevice)
            .mapOrCatch(as: Device?.self, codes: [404], result: nil)
    }

    public func addDevice(device: Device) async throws -> Device {
        try await request(.addDevice(device: device))
            .mapResponse(as: Device.self)
    }

    public func updateDevice(device: Device) async throws -> Device {
        try await request(.updateDevice(device: device))
            .mapResponse(as: Device.self)
    }

    public func isDeviceRegistered() async throws -> Bool {
        try await request(.isDeviceRegistered)
            .mapResponse(as: Bool.self)
    }

    public func migrateDevice(request: MigrateDeviceIdRequest) async throws -> Device {
        try await self.request(.migrateDevice(request: request))
            .mapResponse(as: Device.self)
    }

    public func getNodeAuthToken() async throws -> DeviceToken {
        try await request(.getDeviceToken)
            .mapResponse(as: DeviceToken.self)
    }
}

extension GemDeviceService: GemAPISubscriptionService {
    public func getSubscriptions() async throws -> [WalletSubscriptionChains] {
        try await request(.getSubscriptions)
            .mapResponse(as: [WalletSubscriptionChains].self)
    }

    public func addSubscriptions(subscriptions: [WalletSubscription]) async throws {
        try await request(.addSubscriptions(subscriptions: subscriptions))
            .mapResponse(as: Int.self)
    }

    public func deleteSubscriptions(subscriptions: [WalletSubscriptionChains]) async throws {
        try await request(.deleteSubscriptions(subscriptions: subscriptions))
            .mapResponse(as: Int.self)
    }
}
