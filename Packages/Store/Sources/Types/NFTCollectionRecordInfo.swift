// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GRDB
import Primitives

struct NFTCollectionRecordInfo: Codable, FetchableRecord {
    let collection: NFTCollectionRecord
    let assets: [NFTAssetRecord]
}

extension NFTCollectionRecordInfo {
    func mapToNFTData() -> NFTData {
        return NFTData(
            collection: NFTCollection(
                id: collection.id,
                name: collection.name,
                description: collection.description,
                chain: collection.chain,
                contractAddress: collection.contractAddress,
                images: NFTImages(
                    preview: NFTResource(
                        url: collection.previewImageUrl,
                        mimeType: collection.previewImageMimeType
                    )
                ),
                status: collection.status,
                links: collection.links ?? []
            ),
            assets: assets.map { $0.mapToAsset() }
        )
    }
}
