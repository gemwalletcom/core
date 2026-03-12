// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import GRDB

struct NFTCollectionRecord: Codable, FetchableRecord, PersistableRecord {
    static let databaseTableName = "nft_collections"

    struct Columns {
        static let walletId = Column("walletId")
        static let id = Column("id")
        static let name = Column("name")
        static let description = Column("description")
        static let chain = Column("chain")
        static let contractAddress = Column("contractAddress")
        static let status = Column("status")
        static let links = Column("links")
        static let previewImageUrl = Column("previewImageUrl")
        static let previewImageMimeType = Column("previewImageMimeType")
    }

    var id: String
    var name: String
    var description: String?
    var chain: Chain
    var contractAddress: String
    var status: VerificationStatus
    var links: [AssetLink]?
    
    var previewImageUrl: String
    var previewImageMimeType: String

    static let assets = hasMany(NFTAssetRecord.self).forKey("assets")
}

extension NFTCollectionRecord: CreateTable {

    static func create(db: Database) throws {
        try db.create(table: Self.databaseTableName, ifNotExists: true) {
            $0.column(Columns.id.name, .text)
                .primaryKey()
            $0.column(Columns.name.name, .text).notNull()
            $0.column(Columns.description.name, .text)
            $0.column(Columns.chain.name, .text).notNull()
                .notNull()
                .indexed()
                .references(AssetRecord.databaseTableName, onDelete: .cascade, onUpdate: .cascade)
            $0.column(Columns.contractAddress.name, .text).notNull()
            $0.column(Columns.status.name, .text)
                .notNull()
                .defaults(to: VerificationStatus.unverified.rawValue)
            $0.column(Columns.links.name, .jsonText)
            $0.column(Columns.previewImageUrl.name, .text)
                .notNull()
            $0.column(Columns.previewImageMimeType.name, .text)
                .notNull()
        }
    }
}

extension NFTCollection {
    func record() -> NFTCollectionRecord {
        NFTCollectionRecord(
            id: id,
            name: name,
            description: description,
            chain: chain,
            contractAddress: contractAddress,
            status: status,
            links: links,
            previewImageUrl: images.preview.url,
            previewImageMimeType: images.preview.mimeType
        )
    }
}
