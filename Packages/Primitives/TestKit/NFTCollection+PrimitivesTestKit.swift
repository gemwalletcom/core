// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

public extension NFTCollection {
    static func mock(
        id: String = "test-collection-id",
        name: String = "Test Collection",
        description: String? = "Test Collection Description",
        chain: Chain = .mock(),
        contractAddress: String = "0x123456789abcdef",
        images: NFTImages = NFTImages(preview: .mock()),
        status: VerificationStatus = .verified,
        links: [AssetLink] = []
    ) -> NFTCollection {
        NFTCollection(
            id: id,
            name: name,
            description: description,
            chain: chain,
            contractAddress: contractAddress,
            images: images,
            status: status,
            links: links
        )
    }
}
