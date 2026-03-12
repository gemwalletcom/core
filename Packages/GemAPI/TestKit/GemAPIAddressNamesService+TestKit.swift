// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GemAPI
import Primitives

public struct GemAPIAddressNamesServiceMock: GemAPIAddressNamesService {
    private let addressNames: [AddressName]
    private let error: Error?

    public init(addressNames: [AddressName] = [], error: Error? = nil) {
        self.addressNames = addressNames
        self.error = error
    }

    public func getAddressNames(requests: [ChainAddress]) async throws -> [AddressName] {
        if let error {
            throw error
        }

        return addressNames.filter { address in
            requests.contains {
                $0.chain == address.chain && $0.address == address.address
            }
        }
    }
}
