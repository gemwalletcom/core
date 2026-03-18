// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import Preferences
import BannerService
import DeviceService
import NotificationService
import ChainService

public final class OnstartWalletService: Sendable {

    private let preferences: Preferences
    private let deviceService: any DeviceServiceable
    private let bannerSetupService: BannerSetupService
    private let addressStatusService: AddressStatusService
    private let pushNotificationEnablerService: PushNotificationEnablerService

    public init(
        preferences: Preferences,
        deviceService: any DeviceServiceable,
        bannerSetupService: BannerSetupService,
        addressStatusService: AddressStatusService,
        pushNotificationEnablerService: PushNotificationEnablerService
    ) {
        self.preferences = preferences
        self.deviceService = deviceService
        self.bannerSetupService = bannerSetupService
        self.addressStatusService = addressStatusService
        self.pushNotificationEnablerService = pushNotificationEnablerService
    }

    public func setup(wallet: Wallet) {
        Task {
            try bannerSetupService.setupWallet(wallet: wallet)
            await runAddressStatusCheck(wallet)
        }
    }
    
    public func requestPushPermissions() async {
        do {
            let status = try await pushNotificationEnablerService.getNotificationSettingsStatus()

            switch status {
            case .notDetermined:
                let isEnabled = try await pushNotificationEnablerService.requestPermissions()
                if isEnabled {
                    try await deviceService.update()
                }
            case .authorized, .ephemeral, .provisional, .denied:
                return
            @unknown default:
                return
            }
        } catch {
            debugLog("requestPushPermissions error: \(error)")
        }
    }

    private func runAddressStatusCheck(_ wallet: Wallet) async {
        let walletPreferences = WalletPreferences(walletId: wallet.walletId)
        guard !walletPreferences.completeInitialAddressStatus else { return }

        do {
            let results = try await addressStatusService.getAddressStatus(accounts: wallet.accounts)

            for (account, statuses) in results {
                if statuses.contains(.multiSignature) {
                    try bannerSetupService.setupAccountMultiSignatureWallet(walletId: wallet.walletId, chain: account.chain)
                }
            }
            walletPreferences.completeInitialAddressStatus = true
        } catch {
            debugLog("runAddressStatusCheck error: \(error)")
        }
    }
}
