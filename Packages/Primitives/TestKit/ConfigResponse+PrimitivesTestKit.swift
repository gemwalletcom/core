// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

public extension ConfigResponse {
    static func mock(
        releases: [Release] = [.mock()],
        versions: ConfigVersions = .mock()
    ) -> Self {
        ConfigResponse(
            releases: releases,
            versions: versions,
            swap: SwapConfig(enabledProviders: [])
        )
    }
}

public extension Release {
    static func mock(
        version: String = "99.0",
        upgradeRequired: Bool = false
    ) -> Self {
        Release(
            version: version,
            store: .appStore,
            upgradeRequired: upgradeRequired
        )
    }
}

public extension ConfigVersions {
    static func mock(
        fiatOnRampAssets: Int32 = 1,
        fiatOffRampAssets: Int32 = 1,
        swapAssets: Int32 = 1
    ) -> Self {
        ConfigVersions(
            fiatOnRampAssets: fiatOnRampAssets,
            fiatOffRampAssets: fiatOffRampAssets,
            swapAssets: swapAssets
        )
    }
}
