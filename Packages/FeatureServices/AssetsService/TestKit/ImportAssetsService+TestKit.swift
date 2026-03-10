// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import AssetsService
import GemAPI
import GemAPITestKit
import Preferences
import PreferencesTestKit
import Store
import StoreTestKit

public extension ImportAssetsService {
    static func mock(
        assetListService: any GemAPIAssetsListService = GemAPIAssetsListServiceMock(),
        assetsService: AssetsService = .mock(),
        assetStore: AssetStore = .mock(),
        preferences: Preferences = .mock()
    ) -> ImportAssetsService {
        ImportAssetsService(
            assetListService: assetListService,
            assetsService: assetsService,
            assetStore: assetStore,
            preferences: preferences
        )
    }
}
