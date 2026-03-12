// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GemAPI
import Primitives
import Store
import UIKit
import Preferences

public struct DeviceService: DeviceServiceable {

    public static let authTokenRefreshInterval: Duration = .seconds(300)

    private let deviceProvider: any GemAPIDeviceService
    private let subscriptionsService: SubscriptionService
    private let preferences: Preferences
    private let securePreferences: SecurePreferences
    private static let serialExecutor = SerialExecutor()
    
    public init(
        deviceProvider: any GemAPIDeviceService,
        subscriptionsService: SubscriptionService,
        preferences: Preferences = .standard,
        securePreferences: SecurePreferences = SecurePreferences()
    ) {
        self.deviceProvider = deviceProvider
        self.subscriptionsService = subscriptionsService
        self.preferences = preferences
        self.securePreferences = securePreferences
    }
    
    @discardableResult
    private static func getOrCreateDeviceId(securePreferences: SecurePreferences) throws -> String {
        if let deviceId = try securePreferences.get(key: .deviceId) {
            return deviceId
        }
        let keyPair = try getOrCreateKeyPair(securePreferences: securePreferences)
        let newDeviceId = keyPair.publicKey.hex
        return try securePreferences.set(value: newDeviceId, key: .deviceId)
    }

    @discardableResult
    public static func getOrCreateKeyPair(securePreferences: SecurePreferences) throws -> (privateKey: Data, publicKey: Data) {
        if let privateKey = try securePreferences.getData(key: .devicePrivateKey),
           let publicKey = try securePreferences.getData(key: .devicePublicKey) {
            return (privateKey, publicKey)
        }
        let keyPair = DeviceKeyPair()
        let publicKey = try securePreferences.set(value: keyPair.publicKey, key: .devicePublicKey)
        let privateKey = try securePreferences.set(value: keyPair.privateKey, key: .devicePrivateKey)
        return (privateKey, publicKey)
    }

    public func update() async throws  {
        try await Self.serialExecutor.execute {
            try await migrateDeviceIfNeeded()
            try await updateDevice()
        }
        try? await updateAuthTokenIfNeeded()
    }

    private func migrateDeviceIfNeeded() async throws {
        let deviceId = try getOrCreateDeviceId()
        guard deviceId.count < 64 else { return }

        let keyPair = try getOrCreateKeyPair()
        let publicKey = keyPair.publicKey.hex

        let request = MigrateDeviceIdRequest(oldDeviceId: deviceId, publicKey: publicKey)
        _ = try await deviceProvider.migrateDevice(request: request)

        try securePreferences.set(value: publicKey, key: .deviceId)
    }

    private func updateDevice() async throws {
        let deviceId = try self.getOrCreateDeviceId()
        var device = try await self.getOrCreateDevice(deviceId)
        let localDevice = try await self.currentDevice(deviceId: deviceId)

        let needsSubscriptionUpdate = device.subscriptionsVersion != localDevice.subscriptionsVersion || self.preferences.subscriptionsVersionHasChange
        let needsDeviceUpdate = device != localDevice

        if needsSubscriptionUpdate {
            try await self.subscriptionsService.update()
        }

        if needsSubscriptionUpdate || needsDeviceUpdate {
            device = try await self.updateDevice(localDevice)
        }
    }

    public func updateAuthTokenIfNeeded() async throws {
        try await Self.serialExecutor.execute {
            guard preferences.isDeviceRegistered, shouldUpdateAuthToken() else { return }
            let token = try await deviceProvider.getDeviceToken()
            try securePreferences.setAuthToken(token)
        }
    }

    private func shouldUpdateAuthToken() -> Bool {
        guard let token = try? securePreferences.authToken() else { return true }
        let now = UInt64(Date.now.timeIntervalSince1970)
        let remainingTime = token.expiresAt > now ? token.expiresAt - now : 0
        return remainingTime < UInt64(Self.authTokenRefreshInterval.components.seconds)
    }
    
    private func getOrCreateDevice(_ deviceId: String) async throws -> Device {
        var shouldFetchDevice = preferences.isDeviceRegistered
        if !shouldFetchDevice {
            shouldFetchDevice = try await deviceProvider.isDeviceRegistered()
        }

        if shouldFetchDevice {
            if let device = try await getDevice() {
                preferences.isDeviceRegistered = true
                return device
            }
            preferences.isDeviceRegistered = false
        }

        let device = try await currentDevice(deviceId: deviceId, ignoreSubscriptionsVersion: true)
        let result = try await addDevice(device)
        preferences.isDeviceRegistered = true
        return result
    }
    
    public func getDeviceId() throws -> String {
        try securePreferences.getDeviceId()
    }
    
    public func getSubscriptionsDeviceId() async throws -> String {
        if preferences.subscriptionsVersionHasChange {
            try await update()
        }
        return try getDeviceId()
    }
    
    private func getOrCreateDeviceId() throws -> String {
        try Self.getOrCreateDeviceId(securePreferences: securePreferences)
    }

    @discardableResult
    private func getOrCreateKeyPair() throws -> (privateKey: Data, publicKey: Data) {
        try Self.getOrCreateKeyPair(securePreferences: securePreferences)
    }

    @MainActor
    private func currentDevice(
        deviceId: String,
        ignoreSubscriptionsVersion: Bool = false
    ) throws -> Device {
        let deviceToken = try securePreferences.get(key: .deviceToken) ?? .empty
        let locale = Locale.current.usageLanguageIdentifier()
        #if targetEnvironment(simulator)
        let platformStore = PlatformStore.local
        #else
        let platformStore = PlatformStore.appStore
        #endif
        
        return Device(
            id: deviceId,
            platform: .ios,
            platformStore: platformStore,
            os: UIDevice.current.osName,
            model: UIDevice.current.modelName,
            token: deviceToken,
            locale: locale,
            version: Bundle.main.releaseVersionNumber,
            currency: preferences.currency,
            isPushEnabled: preferences.isPushNotificationsEnabled,
            isPriceAlertsEnabled: preferences.isPriceAlertsEnabled,
            subscriptionsVersion: ignoreSubscriptionsVersion ? 0 : preferences.subscriptionsVersion.asInt32
        )
    }

    private func getDevice() async throws -> Device? {
        try await deviceProvider.getDevice()
    }

    @discardableResult
    private func addDevice(_ device: Device) async throws -> Device {
        try await deviceProvider.addDevice(device: device)
    }

    @discardableResult
    private func updateDevice(_ device: Device) async throws -> Device {
        try await deviceProvider.updateDevice(device: device)
    }
}
