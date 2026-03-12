// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import GRDB

struct NodeSelectedRecord: Codable, FetchableRecord, PersistableRecord, TableRecord {
    static let databaseTableName: String = "nodes_selected"

    enum Columns {
        static let chain = Column("chain")
        static let nodeUrl = Column("nodeUrl")
    }

    var chain: Chain
    var nodeUrl: String

}

extension NodeSelectedRecord: CreateTable {
    static func create(db: Database) throws {
        try db.create(table: Self.databaseTableName, ifNotExists: true) {
            $0.column(Columns.chain.name, .text).primaryKey()
                .references(AssetRecord.databaseTableName, onDelete: .cascade, onUpdate: .cascade)
            $0.column(Columns.nodeUrl.name, .text).notNull()
        }
    }
}
