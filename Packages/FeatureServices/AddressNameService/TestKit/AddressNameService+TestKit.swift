// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import AddressNameService
import GemAPITestKit
import Store
import StoreTestKit

public extension AddressNameService {
    static func mock(
        addressStore: AddressStore = .mock(),
        apiService: GemAPIAddressNamesServiceMock = GemAPIAddressNamesServiceMock()
    ) -> AddressNameService {
        AddressNameService(addressStore: addressStore, apiService: apiService)
    }
}
