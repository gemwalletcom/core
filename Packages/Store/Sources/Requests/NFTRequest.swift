// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GRDB
import Primitives

public enum NFTFilter: Sendable, Hashable {
    case all
    case unverified
    case collection(id: String)
}

public struct NFTRequest: DatabaseQueryable {

    private let walletId: WalletId
    private let filter: NFTFilter

    public init(walletId: WalletId, filter: NFTFilter) {
        self.walletId = walletId
        self.filter = filter
    }

    public func fetch(_ db: Database) throws -> [NFTData] {
        var request = NFTCollectionRecord
            .including(
                all: NFTCollectionRecord.assets
                    .joining(
                        required: NFTAssetRecord.assetAssociations
                            .filter(NFTAssetAssociationRecord.Columns.walletId == walletId.id)
                    )
            )
            .distinct()
            .asRequest(of: NFTCollectionRecordInfo.self)

        switch filter {
        case .all: break
        case .unverified: request = request.filter(NFTCollectionRecord.Columns.status != VerificationStatus.verified.rawValue)
        case .collection(let id): request = request.filter(NFTCollectionRecord.Columns.id == id)
        }

        return try request
            .fetchAll(db)
            .map { $0.mapToNFTData() }
            .filter { $0.assets.isNotEmpty }
    }
}

extension NFTRequest: Equatable {}
