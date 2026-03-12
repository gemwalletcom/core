// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Testing
import Primitives
import PrimitivesTestKit
import Preferences
import PreferencesTestKit
import GemAPITestKit
import AppServiceTestKit
@testable import AppService

struct AppReleaseServiceTests {
    @Test
    func newestRelease() async {
        let configPreferences = ConfigPreferences.mock()
        configPreferences.config = .mock()
        let service = AppReleaseService(configService: ConfigService(configPreferences: configPreferences, apiService: GemAPIConfigServiceMock(config: .mock())))

        #expect(await service.getNewestRelease()?.version == "99.0")
    }

    @Test
    func releaseFromConfig() {
        #expect(AppReleaseService(configService: .mock()).release(ConfigResponse.mock())?.version == "99.0")
    }
}
