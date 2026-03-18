// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GemAPI
import Primitives
import PrimitivesTestKit

public actor GemAPIDeviceServiceMock: GemAPIDeviceService {
    private let delay: Duration?
    private let isDeviceRegisteredResult: Bool
    private let getDeviceResult: Device?
    private let nodeAuthToken: DeviceToken

    public private(set) var isDeviceRegisteredCalls = 0
    public private(set) var getDeviceCalls = 0
    public private(set) var addDeviceCalls = 0
    public private(set) var updateDeviceCalls = 0
    public private(set) var migrateDeviceCalls = 0
    public private(set) var getNodeAuthTokenCalls = 0

    public init(
        delay: Duration? = nil,
        isDeviceRegistered: Bool = true,
        getDeviceResult: Device? = Device.mock(),
        nodeAuthToken: DeviceToken = .init(token: "", expiresAt: 0)
    ) {
        self.delay = delay
        self.isDeviceRegisteredResult = isDeviceRegistered
        self.getDeviceResult = getDeviceResult
        self.nodeAuthToken = nodeAuthToken
    }

    public func getDevice() async throws -> Device? {
        getDeviceCalls += 1
        return getDeviceResult
    }

    public func addDevice(device: Device) async throws -> Device {
        addDeviceCalls += 1
        try await sleepIfNeeded()
        return device
    }

    public func updateDevice(device: Device) async throws -> Device {
        updateDeviceCalls += 1
        try await sleepIfNeeded()
        return device
    }

    public func isDeviceRegistered() async throws -> Bool {
        isDeviceRegisteredCalls += 1
        return isDeviceRegisteredResult
    }

    public func migrateDevice(request: MigrateDeviceIdRequest) async throws -> Device {
        migrateDeviceCalls += 1
        return Device.mock()
    }

    public func getNodeAuthToken() async throws -> DeviceToken {
        getNodeAuthTokenCalls += 1
        return nodeAuthToken
    }

    private func sleepIfNeeded() async throws {
        if let delay {
            try await Task.sleep(for: delay)
        }
    }
}
