// Copyright (c). Gem Wallet. All rights reserved.

import Testing
import PreferencesTestKit
import Primitives

@testable import Preferences
import Foundation

struct PreferencesTests {
    private let preferences: Preferences = .mock()

    @Test
    func testDefaultPreferences() {
        #expect(preferences.currency == Currency.usd.rawValue)

        #expect(preferences.importFiatMappingsVersion == 0)
        #expect(preferences.importFiatPurchaseAssetsVersion == 0)
        #expect(preferences.localAssetsVersion == 0)
        #expect(preferences.fiatOnRampAssetsVersion == 0)
        #expect(preferences.fiatOffRampAssetsVersion == 0)
        #expect(preferences.swapAssetsVersion == 0)
        #expect(preferences.launchesCount == 0)
        #expect(preferences.subscriptionsVersion == 0)
        #expect(preferences.authenticationLockOption == 0)

        #expect(preferences.currentWalletId == nil)
        #expect(preferences.isSubscriptionsEnabled)
        #expect(!preferences.isPushNotificationsEnabled)
        #expect(!preferences.isPriceAlertsEnabled)
        #expect(!preferences.rateApplicationShown)
        #expect(!preferences.isDeveloperEnabled)
        #expect(!preferences.isHideBalanceEnabled)
        #expect(preferences.skippedReleaseVersion == nil)
        #expect(preferences.perpetualLeverage == 10)
    }

    @Test
    func testIncrementLaunchesCount() {
        #expect(preferences.launchesCount == 0)
        preferences.incrementLaunchesCount()
        #expect(preferences.launchesCount == 1)
    }

    @Test
    func invalidateSubscriptionsIsIdempotent() {
        preferences.subscriptionsVersion = 8
        preferences.subscriptionsVersionHasChange = false

        preferences.invalidateSubscriptions()
        let version = preferences.subscriptionsVersion
        preferences.invalidateSubscriptions()

        #expect(version == 9)
        #expect(preferences.subscriptionsVersion == version)
        #expect(preferences.subscriptionsVersionHasChange)
    }

    @Test
    func testUpdatePreferences() {
        preferences.currency = Currency.eur.rawValue
        #expect(preferences.currency == Currency.eur.rawValue)

        preferences.importFiatMappingsVersion = 1
        #expect(preferences.importFiatMappingsVersion == 1)

        preferences.importFiatPurchaseAssetsVersion = 2
        #expect(preferences.importFiatPurchaseAssetsVersion == 2)

        preferences.localAssetsVersion = 3
        #expect(preferences.localAssetsVersion == 3)

        preferences.fiatOnRampAssetsVersion = 4
        #expect(preferences.fiatOnRampAssetsVersion == 4)

        preferences.fiatOffRampAssetsVersion = 5
        #expect(preferences.fiatOffRampAssetsVersion == 5)

        preferences.swapAssetsVersion = 6
        #expect(preferences.swapAssetsVersion == 6)

        preferences.launchesCount = 7
        #expect(preferences.launchesCount == 7)

        preferences.subscriptionsVersion = 8
        #expect(preferences.subscriptionsVersion == 8)

        preferences.authenticationLockOption = 9
        #expect(preferences.authenticationLockOption == 9)

        preferences.currentWalletId = "wallet123"
        #expect(preferences.currentWalletId == "wallet123")

        preferences.isSubscriptionsEnabled = false
        #expect(!preferences.isSubscriptionsEnabled)

        preferences.isPushNotificationsEnabled = true
        #expect(preferences.isPushNotificationsEnabled)

        preferences.isPriceAlertsEnabled = true
        #expect(preferences.isPriceAlertsEnabled)

        preferences.rateApplicationShown = true
        #expect(preferences.rateApplicationShown)

        preferences.isDeveloperEnabled = true
        #expect(preferences.isDeveloperEnabled)

        preferences.isHideBalanceEnabled = true
        #expect(preferences.isHideBalanceEnabled)

        preferences.setExplorerName(chain: .bitcoin, name: "btc")
        #expect(preferences.explorerName(chain: .bitcoin) == "btc")
        
        preferences.skippedReleaseVersion = "1.2.3"
        #expect(preferences.skippedReleaseVersion == "1.2.3")

        preferences.perpetualLeverage = 25
        #expect(preferences.perpetualLeverage == 25)
    }

    @Test
    func testClear() {
        preferences.currency = Currency.eur.rawValue
        preferences.importFiatMappingsVersion = 1
        preferences.importFiatPurchaseAssetsVersion = 2
        preferences.localAssetsVersion = 3
        preferences.fiatOnRampAssetsVersion = 4
        preferences.fiatOffRampAssetsVersion = 5
        preferences.swapAssetsVersion = 6
        preferences.launchesCount = 7
        preferences.subscriptionsVersion = 8
        preferences.authenticationLockOption = 9
        preferences.currentWalletId = "wallet123"
        preferences.isSubscriptionsEnabled = false
        preferences.isPushNotificationsEnabled = true
        preferences.isPriceAlertsEnabled = true
        preferences.rateApplicationShown = true
        preferences.isDeveloperEnabled = true
        preferences.isHideBalanceEnabled = true
        preferences.setExplorerName(chain: .bitcoin, name: "btc")
        preferences.skippedReleaseVersion = "1.0.0"
        preferences.perpetualLeverage = 25

        #expect(preferences.currency == Currency.eur.rawValue)
        #expect(preferences.importFiatMappingsVersion == 1)
        #expect(preferences.importFiatPurchaseAssetsVersion == 2)
        #expect(preferences.localAssetsVersion == 3)
        #expect(preferences.fiatOnRampAssetsVersion == 4)
        #expect(preferences.fiatOffRampAssetsVersion == 5)
        #expect(preferences.swapAssetsVersion == 6)
        #expect(preferences.launchesCount == 7)
        #expect(preferences.subscriptionsVersion == 8)
        #expect(preferences.authenticationLockOption == 9)
        #expect(preferences.currentWalletId == "wallet123")
        #expect(!preferences.isSubscriptionsEnabled)
        #expect(preferences.isPushNotificationsEnabled)
        #expect(preferences.isPriceAlertsEnabled)
        #expect(preferences.rateApplicationShown)
        #expect(preferences.isDeveloperEnabled)
        #expect(preferences.isHideBalanceEnabled)
        #expect(preferences.explorerName(chain: .bitcoin) == "btc")
        #expect(preferences.skippedReleaseVersion == "1.0.0")

        preferences.clear()

        #expect(preferences.currency == Currency.usd.rawValue)
        #expect(preferences.importFiatMappingsVersion == 0)
        #expect(preferences.importFiatPurchaseAssetsVersion == 0)
        #expect(preferences.localAssetsVersion == 0)
        #expect(preferences.fiatOnRampAssetsVersion == 0)
        #expect(preferences.fiatOffRampAssetsVersion == 0)
        #expect(preferences.swapAssetsVersion == 0)
        #expect(preferences.launchesCount == 0)
        #expect(preferences.subscriptionsVersion == 0)
        #expect(preferences.authenticationLockOption == 0)
        #expect(preferences.currentWalletId == nil)
        #expect(preferences.isSubscriptionsEnabled)
        #expect(!preferences.isPushNotificationsEnabled)
        #expect(!preferences.isPriceAlertsEnabled)
        #expect(!preferences.rateApplicationShown)
        #expect(!preferences.isDeveloperEnabled)
        #expect(!preferences.isHideBalanceEnabled)
        #expect(preferences.explorerName(chain: .bitcoin) == nil)
        #expect(preferences.skippedReleaseVersion == nil)
        #expect(preferences.perpetualLeverage == 10)

    }

    @Test
    func testReinitializeReflectsExternalChanges() {
        let testDefaults = UserDefaults(suiteName: "testReinitialize")!
        testDefaults.removePersistentDomain(forName: "testReinitialize")

        let preferences = Preferences(defaults: testDefaults)

        #expect(preferences.currency == Currency.usd.rawValue)
        #expect(preferences.launchesCount == 0)
        #expect(preferences.currentWalletId == nil)

        testDefaults.set(Currency.eur.rawValue, forKey: "currency")
        testDefaults.set(5, forKey: "launches_count")
        testDefaults.set("walletXYZ", forKey: "currentWallet")

        #expect(preferences.currency == Currency.eur.rawValue)
        #expect(preferences.launchesCount == 5)
        #expect(preferences.currentWalletId == "walletXYZ")

        let newPrefs = Preferences(defaults: testDefaults)
        #expect(newPrefs.currency == Currency.eur.rawValue)
        #expect(newPrefs.launchesCount == 5)
        #expect(newPrefs.currentWalletId == "walletXYZ")

        testDefaults.removePersistentDomain(forName: "testReinitialize")
    }

    @Test
    func testOptionalNilAssignment() {
        preferences.currentWalletId = "wallet123"
        #expect(preferences.currentWalletId == "wallet123")

        preferences.currentWalletId = nil
        #expect(preferences.currentWalletId == nil)
    }
}
