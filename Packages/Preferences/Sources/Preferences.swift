// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

public final class Preferences: @unchecked Sendable {
    public enum Constants {
        public static let appGroupIdentifier = "group.com.gemwallet.ios"
    }

    public struct Keys {
        static let currency = "currency"
        static let importFiatMappingsVersion = "migrate_fiat_mappings_version"
        static let importFiatPurchaseAssetsVersion = "migrate_fiat_purchase_assets_version"
        static let localAssetsVersion = "local_assets_version"
        static let fiatOnRampAssetsVersion = "fiat_on_ramp_assets_version"
        static let fiatOffRampAssetsVersion = "fiat_off_ramp_assets_version"
        static let swapAssetsVersion = "swap_assets_version"
        static let launchesCount = "launches_count"
        static let subscriptionsVersion = "subscriptions_version"
        static let subscriptionsVersionHasChange = "subscriptions_version_has_change"
        static let currentWalletId = "currentWallet"
        static let isPushNotificationsEnabled = "is_push_notifications_enabled"
        static let isPriceAlertsEnabled = "is_price_alerts_enabled"
        static let isSubscriptionsEnabled = "is_subscriptions_enabled"
        static let rateApplicationShown = "rate_application_shown"
        static let authenticationLockOption = "authentication_lock_option"
        static let isDeveloperEnabled = "is_developer_enabled"
        static let isHideBalanceEnabled = "is_balance_privacy_enabled"
        static let isAcceptTermsCompleted = "is_accepted_terms"
        static let skippedReleaseVersion = "skipped_release_version"
        static let isWalletConnectActivated = "is_walletconnect_activated"
        static let perpetualsMarketsUpdatedAt = "perpetual_markets_updated_at"
        static let perpetualPricesUpdatedAt = "perpetual_prices_updated_at"
        static let isPerpetualEnabled = "is_perpetual_enabled"
        static let perpetualLeverage = "perpetual_leverage"
        static let isDeviceRegistered = "is_device_registered"
    }

    @ConfigurableDefaults(key: Keys.currency, defaultValue: Currency.usd.rawValue)
    public var currency: String

    @ConfigurableDefaults(key: Keys.importFiatMappingsVersion, defaultValue: 0)
    public var importFiatMappingsVersion: Int

    @ConfigurableDefaults(key: Keys.importFiatPurchaseAssetsVersion, defaultValue: 0)
    public var importFiatPurchaseAssetsVersion: Int

    @ConfigurableDefaults(key: Keys.localAssetsVersion, defaultValue: 0)
    public var localAssetsVersion: Int

    @ConfigurableDefaults(key: Keys.fiatOnRampAssetsVersion, defaultValue: 0)
    public var fiatOnRampAssetsVersion: Int

    @ConfigurableDefaults(key: Keys.fiatOffRampAssetsVersion, defaultValue: 0)
    public var fiatOffRampAssetsVersion: Int

    @ConfigurableDefaults(key: Keys.swapAssetsVersion, defaultValue: 0)
    public var swapAssetsVersion: Int

    @ConfigurableDefaults(key: Keys.launchesCount, defaultValue: 0)
    public var launchesCount: Int

    @ConfigurableDefaults(key: Keys.subscriptionsVersion, defaultValue: 0)
    public var subscriptionsVersion: Int

    @ConfigurableDefaults(key: Keys.subscriptionsVersionHasChange, defaultValue: true)
    public var subscriptionsVersionHasChange: Bool

    @ConfigurableDefaults(key: Keys.currentWalletId, defaultValue: .none)
    public var currentWalletId: String?

    @ConfigurableDefaults(key: Keys.isPushNotificationsEnabled, defaultValue: false)
    public var isPushNotificationsEnabled: Bool

    @ConfigurableDefaults(key: Keys.isPriceAlertsEnabled, defaultValue: false)
    public var isPriceAlertsEnabled: Bool

    @ConfigurableDefaults(key: Keys.isSubscriptionsEnabled, defaultValue: true)
    public var isSubscriptionsEnabled: Bool

    @ConfigurableDefaults(key: Keys.rateApplicationShown, defaultValue: false)
    public var rateApplicationShown: Bool

    @ConfigurableDefaults(key: Keys.authenticationLockOption, defaultValue: 0)
    public var authenticationLockOption: Int

    @ConfigurableDefaults(key: Keys.isDeveloperEnabled, defaultValue: false)
    public var isDeveloperEnabled: Bool

    @ConfigurableDefaults(key: Keys.isHideBalanceEnabled, defaultValue: false)
    public var isHideBalanceEnabled: Bool

    @ConfigurableDefaults(key: Keys.isAcceptTermsCompleted, defaultValue: false)
    public var isAcceptTermsCompleted: Bool

    @ConfigurableDefaults(key: Keys.skippedReleaseVersion, defaultValue: nil)
    public var skippedReleaseVersion: String?

    @ConfigurableDefaults(key: Keys.isWalletConnectActivated, defaultValue: nil)
    public var isWalletConnectActivated: Bool?

    @ConfigurableDefaults(key: Keys.perpetualsMarketsUpdatedAt, defaultValue: nil)
    public var perpetualMarketsUpdatedAt: Date?

    @ConfigurableDefaults(key: Keys.perpetualPricesUpdatedAt, defaultValue: nil)
    public var perpetualPricesUpdatedAt: Date?

    @ConfigurableDefaults(key: Keys.isPerpetualEnabled, defaultValue: false)
    public var isPerpetualEnabled: Bool

    @ConfigurableDefaults(key: Keys.perpetualLeverage, defaultValue: 10)
    public var perpetualLeverage: UInt8

    @ConfigurableDefaults(key: Keys.isDeviceRegistered, defaultValue: false)
    public var isDeviceRegistered: Bool

    public static let standard = Preferences()
    private let defaults: UserDefaults

    public init(defaults: UserDefaults = .standard) {
        self.defaults = defaults
        configureAllProperties(with: defaults)
    }

    private func configureAllProperties(with defaults: UserDefaults) {
        let sharedDefaults = UserDefaults(suiteName: Constants.appGroupIdentifier)

        func configure<T>(_ keyPath: ReferenceWritableKeyPath<Preferences, ConfigurableDefaults<T>>, key: String, defaultValue: T, sharedDefaults: UserDefaults? = nil) {
            self[keyPath: keyPath] = ConfigurableDefaults(key: key, defaultValue: defaultValue, defaults: defaults, sharedDefaults: sharedDefaults)
        }
        configure(\._currency, key: Keys.currency, defaultValue: Currency.usd.rawValue, sharedDefaults: sharedDefaults)
        configure(\._importFiatMappingsVersion, key: Keys.importFiatMappingsVersion, defaultValue: 0)
        configure(\._importFiatPurchaseAssetsVersion, key: Keys.importFiatPurchaseAssetsVersion, defaultValue: 0)
        configure(\._localAssetsVersion, key: Keys.localAssetsVersion, defaultValue: 0)
        configure(\._fiatOnRampAssetsVersion, key: Keys.fiatOnRampAssetsVersion, defaultValue: 0)
        configure(\._fiatOffRampAssetsVersion, key: Keys.fiatOffRampAssetsVersion, defaultValue: 0)
        configure(\._swapAssetsVersion, key: Keys.swapAssetsVersion, defaultValue: 0)
        configure(\._launchesCount, key: Keys.launchesCount, defaultValue: 0)
        configure(\._subscriptionsVersion, key: Keys.subscriptionsVersion, defaultValue: 0)
        configure(\._subscriptionsVersionHasChange, key: Keys.subscriptionsVersionHasChange, defaultValue: true)
        configure(\._currentWalletId, key: Keys.currentWalletId, defaultValue: nil)
        configure(\._isPushNotificationsEnabled, key: Keys.isPushNotificationsEnabled, defaultValue: false)
        configure(\._isPriceAlertsEnabled, key: Keys.isPriceAlertsEnabled, defaultValue: false)
        configure(\._isSubscriptionsEnabled, key: Keys.isSubscriptionsEnabled, defaultValue: true)
        configure(\._rateApplicationShown, key: Keys.rateApplicationShown, defaultValue: false)
        configure(\._authenticationLockOption, key: Keys.authenticationLockOption, defaultValue: 0)
        configure(\._isDeveloperEnabled, key: Keys.isDeveloperEnabled, defaultValue: false)
        configure(\._isHideBalanceEnabled, key: Keys.isHideBalanceEnabled, defaultValue: false)
        configure(\._isAcceptTermsCompleted, key: Keys.isAcceptTermsCompleted, defaultValue: false)
        configure(\._skippedReleaseVersion, key: Keys.skippedReleaseVersion, defaultValue: nil)
        configure(\._isWalletConnectActivated, key: Keys.isWalletConnectActivated, defaultValue: nil)
        configure(\._perpetualMarketsUpdatedAt, key: Keys.perpetualsMarketsUpdatedAt, defaultValue: nil)
        configure(\._perpetualPricesUpdatedAt, key: Keys.perpetualPricesUpdatedAt, defaultValue: nil)
        configure(\._isPerpetualEnabled, key: Keys.isPerpetualEnabled, defaultValue: false)
        configure(\._perpetualLeverage, key: Keys.perpetualLeverage, defaultValue: 10)
        configure(\._isDeviceRegistered, key: Keys.isDeviceRegistered, defaultValue: false)
    }

    public func incrementLaunchesCount() {
        launchesCount += 1
    }

    public var hasCurrency: Bool {
        defaults.object(forKey: Keys.currency) != nil
    }

    public func clear() {
        for key in defaults.dictionaryRepresentation().keys {
            defaults.removeObject(forKey: key)
        }
    }

    public func invalidateSubscriptions() {
        guard !subscriptionsVersionHasChange else { return }
        subscriptionsVersion += 1
        subscriptionsVersionHasChange = true
    }

    private struct ExplorerKeys {
        static let explorerName = "explorer_name"
    }

    public func setExplorerName(chain: Chain, name: String) {
        defaults.setValue(name, forKey: "\(ExplorerKeys.explorerName)_\(chain.rawValue)")
    }

    public func explorerName(chain: Chain) -> String? {
        defaults.string(forKey: "\(ExplorerKeys.explorerName)_\(chain.rawValue)")
    }
}
