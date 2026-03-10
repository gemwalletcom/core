// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import NFTService
import GemAPITestKit
import StoreTestKit
import DeviceServiceTestKit

public extension NFTService {
    static func mock() -> NFTService {
        NFTService(
            apiService: GemAPINFTServiceMock(),
            nftStore: .mock(),
            deviceService: DeviceServiceMock()
        )
    }
}
