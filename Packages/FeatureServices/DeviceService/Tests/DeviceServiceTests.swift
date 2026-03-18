// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Testing
import Preferences
import PreferencesTestKit
import StoreTestKit
import GemAPI
import GemAPITestKit
import Primitives
import PrimitivesTestKit

@testable import DeviceService

struct DeviceServiceTests {

    @Test
    func prepareForWalletRequestSkipsCleanState() async throws {
        let preferences = Preferences.mock()
        preferences.isDeviceRegistered = true
        preferences.subscriptionsVersionHasChange = false

        let securePreferences = SecurePreferences.mock()
        try securePreferences.set(value: String(repeating: "a", count: 64), key: .deviceId)

        let deviceProvider = GemAPIDeviceServiceMock(
            isDeviceRegistered: false,
            getDeviceResult: nil
        )
        let service = makeService(
            preferences: preferences,
            deviceProvider: deviceProvider,
            subscriptionProvider: GemAPISubscriptionServiceMock(),
            securePreferences: securePreferences
        )

        try await service.prepareForWalletRequest()

        #expect(await deviceProvider.isDeviceRegisteredCalls == 0)
        #expect(await deviceProvider.getDeviceCalls == 0)
        #expect(await deviceProvider.addDeviceCalls == 0)
        #expect(await deviceProvider.updateDeviceCalls == 0)
    }

    @Test
    func prepareForWalletRequestSharesInFlightSync() async throws {
        let preferences = Preferences.mock()
        preferences.isDeviceRegistered = false
        preferences.subscriptionsVersionHasChange = true

        let deviceProvider = GemAPIDeviceServiceMock(
            delay: .milliseconds(50),
            isDeviceRegistered: false,
            getDeviceResult: nil
        )
        let subscriptionProvider = GemAPISubscriptionServiceMock(delay: .milliseconds(50))
        let service = makeService(
            preferences: preferences,
            deviceProvider: deviceProvider,
            subscriptionProvider: subscriptionProvider
        )

        async let first: Void = service.prepareForWalletRequest()
        async let second: Void = service.prepareForWalletRequest()
        _ = try await (first, second)

        #expect(await deviceProvider.isDeviceRegisteredCalls == 1)
        #expect(await deviceProvider.addDeviceCalls == 1)
        #expect(await deviceProvider.updateDeviceCalls == 1)
        #expect(await subscriptionProvider.getSubscriptionsCalls == 1)
        #expect(!preferences.subscriptionsVersionHasChange)
    }

    @Test
    func prepareForWalletRequestSynchronizesLegacyDeviceId() async throws {
        let preferences = Preferences.mock()
        preferences.isDeviceRegistered = true
        preferences.subscriptionsVersionHasChange = false

        let securePreferences = SecurePreferences.mock()
        try securePreferences.set(value: "legacy-device-id", key: .deviceId)

        let deviceProvider = GemAPIDeviceServiceMock(
            isDeviceRegistered: false,
            getDeviceResult: nil
        )
        let service = makeService(
            preferences: preferences,
            deviceProvider: deviceProvider,
            subscriptionProvider: GemAPISubscriptionServiceMock(),
            securePreferences: securePreferences
        )

        try await service.prepareForWalletRequest()

        #expect(await deviceProvider.migrateDeviceCalls == 1)
        #expect(await deviceProvider.addDeviceCalls == 1)
    }

    @Test
    func updateAndPrepareForWalletRequestShareSyncTask() async throws {
        let preferences = Preferences.mock()
        preferences.isDeviceRegistered = false
        preferences.subscriptionsVersionHasChange = true

        let deviceProvider = GemAPIDeviceServiceMock(
            delay: .milliseconds(50),
            isDeviceRegistered: false,
            getDeviceResult: nil
        )
        let subscriptionProvider = GemAPISubscriptionServiceMock(delay: .milliseconds(50))
        let service = makeService(
            preferences: preferences,
            deviceProvider: deviceProvider,
            subscriptionProvider: subscriptionProvider
        )

        async let update: Void = service.update()
        async let ready: Void = service.prepareForWalletRequest()
        _ = try await (update, ready)

        #expect(await deviceProvider.isDeviceRegisteredCalls == 1)
        #expect(await deviceProvider.addDeviceCalls == 1)
        #expect(await deviceProvider.updateDeviceCalls == 1)
        #expect(await deviceProvider.getNodeAuthTokenCalls == 1)
        #expect(await subscriptionProvider.getSubscriptionsCalls == 1)
    }

    @Test
    func prepareForWalletRequestWaitsForInFlightSyncBeforeFastPath() async throws {
        let preferences = Preferences.mock()
        preferences.isDeviceRegistered = true
        preferences.subscriptionsVersionHasChange = true

        let securePreferences = SecurePreferences.mock()
        try securePreferences.set(value: String(repeating: "a", count: 64), key: .deviceId)

        let deviceProvider = GemAPIDeviceServiceMock(
            delay: .milliseconds(150),
            isDeviceRegistered: true,
            getDeviceResult: Device.mock()
        )
        let service = makeService(
            preferences: preferences,
            deviceProvider: deviceProvider,
            subscriptionProvider: GemAPISubscriptionServiceMock(),
            securePreferences: securePreferences
        )
        let ready = CompletionFlag()

        let updateTask = Task {
            try await service.update()
        }

        try await Task.sleep(for: .milliseconds(50))
        #expect(!preferences.subscriptionsVersionHasChange)

        let readyTask = Task {
            try await service.prepareForWalletRequest()
            await ready.markComplete()
        }

        try await Task.sleep(for: .milliseconds(20))
        #expect(await !ready.isComplete)

        try await updateTask.value
        try await readyTask.value
    }

    @Test
    func prepareForWalletRequestPropagatesSyncErrors() async {
        let preferences = Preferences.mock()
        preferences.isDeviceRegistered = false
        preferences.subscriptionsVersionHasChange = true

        let service = makeService(
            preferences: preferences,
            deviceProvider: GemAPIDeviceServiceMock(
                isDeviceRegistered: false,
                getDeviceResult: nil
            ),
            subscriptionProvider: GemAPISubscriptionServiceMock(getSubscriptionsError: TestError.failed)
        )

        await #expect(throws: TestError.self) {
            try await service.prepareForWalletRequest()
        }

        #expect(preferences.subscriptionsVersionHasChange)
    }
}

private extension DeviceServiceTests {
    func makeService(
        preferences: Preferences,
        deviceProvider: any GemAPIDeviceService,
        subscriptionProvider: any GemAPISubscriptionService,
        securePreferences: SecurePreferences = .mock()
    ) -> DeviceService {
        DeviceService(
            deviceProvider: deviceProvider,
            subscriptionsService: SubscriptionService(
                subscriptionProvider: subscriptionProvider,
                walletStore: .mock(),
                preferences: preferences
            ),
            preferences: preferences,
            securePreferences: securePreferences
        )
    }
}

private enum TestError: Error {
    case failed
}

private actor CompletionFlag {
    private(set) var isComplete = false

    func markComplete() {
        isComplete = true
    }
}
