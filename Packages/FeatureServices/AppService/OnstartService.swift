// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Store
import Primitives
import NodeService
import AssetsService
import GemAPI
import Preferences
import WalletService
import UIKit

// OnstartService runs services before the app starts.
// See OnstartAsyncService for any background tasks to run after start
public struct OnstartService: Sendable {

    private let assetListService: any GemAPIAssetsListService
    private let assetsService: AssetsService
    private let assetStore: AssetStore
    private let nodeStore: NodeStore
    private let preferences: Preferences
    private let walletService: WalletService

    public init(
        assetListService: any GemAPIAssetsListService,
        assetsService: AssetsService,
        assetStore: AssetStore,
        nodeStore: NodeStore,
        preferences: Preferences,
        walletService: WalletService
    ) {
        self.assetListService = assetListService
        self.assetsService = assetsService
        self.assetStore = assetStore
        self.nodeStore = nodeStore
        self.preferences = preferences
        self.walletService = walletService
    }

    @MainActor
    public func configure() {
        validateDeviceSecurity()
        configureURLCache()
        do {
            try excludeDirectoriesFromBackup()
            try migrations()
        } catch {
            debugLog("configure error: \(error)")
        }
        preferences.incrementLaunchesCount()

        #if DEBUG
        configureScreenshots()
        #endif
    }
}

// MARK: - Private

extension OnstartService {

    private func migrations() throws {
        try walletService.setup(chains: AssetConfiguration.allChains)
        try ImportAssetsService(
            assetListService: assetListService,
            assetsService: assetsService,
            assetStore: assetStore,
            preferences: preferences
        ).migrate()

        if !preferences.hasCurrency, let currency = Locale.current.currency {
            preferences.currency = (Currency(rawValue: currency.identifier) ?? .usd).rawValue
        }
    }

    private func configureURLCache() {
        URLCache.shared.memoryCapacity = 256_000_000 // ~256 MB memory space
        URLCache.shared.diskCapacity = 1_000_000_000 // ~1GB disk cache space
    }

    private func excludeDirectoriesFromBackup() throws {
        let excludedBackupDirectories: [FileManager.Directory] = [.documents, .applicationSupport, .library(.preferences)]
        for directory in excludedBackupDirectories {
            try FileManager.default.addSkipBackupAttributeToItemAtURL(directory.url)

            #if DEBUG
            debugLog("Excluded backup directory: \(directory.directory)")
            #endif
        }
    }

    @MainActor
    private func validateDeviceSecurity() {
        let device = UIDevice.current
        if !device.isSimulator && (device.isJailBroken || device.isFridaDetected) {
            fatalError()
        }
    }

    private func configureScreenshots() {
        if ProcessInfo.processInfo.environment["SCREENSHOTS_PATH"] != nil {
            if let currency = Locale.current.currency, let currency = Currency(rawValue: currency.identifier) {
                Preferences.standard.currency = currency.rawValue
            } else {
                Preferences.standard.currency = Currency.usd.rawValue
            }
        }
    }
}
