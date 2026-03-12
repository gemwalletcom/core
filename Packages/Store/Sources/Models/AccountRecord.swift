// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import GRDB

struct AccountRecord: Codable, FetchableRecord, PersistableRecord  {
    
    static let databaseTableName: String = "wallets_accounts"

    enum Columns {
        static let walletId = Column("walletId")
        static let chain = Column("chain")
        static let address = Column("address")
        static let index = Column("index")
        static let extendedPublicKey = Column("extendedPublicKey")
        static let derivationPath = Column("derivationPath")
    }

    var walletId: String
    var chain: Chain
    var address: String
    var extendedPublicKey: String?
    var index: Int
    var derivationPath: String
}

extension AccountRecord: CreateTable {
    static func create(db: Database) throws {
        try db.create(table: Self.databaseTableName, ifNotExists: true) {
            $0.column(Columns.walletId.name, .text)
                .notNull()
                .indexed()
                .references(WalletRecord.databaseTableName, onDelete: .cascade, onUpdate: .cascade)
            $0.column(Columns.chain.name, .text)
                .notNull()
                .references(AssetRecord.databaseTableName, onDelete: .cascade, onUpdate: .cascade)
            $0.column(Columns.address.name, .text)
                .notNull()
            $0.column(Columns.extendedPublicKey.name, .text)
            $0.column(Columns.index.name, .numeric)
                .defaults(to: 0)
                .notNull()
            $0.column(Columns.derivationPath.name, .text)
                .notNull()
            $0.uniqueKey([
                Columns.walletId.name,
                Columns.chain.name,
                Columns.derivationPath.name,
                Columns.address.name
            ])
        }
    }
}

extension AccountRecord {
    func mapToAccount() -> Account {
        return Account(
            chain: chain,
            address: address,
            derivationPath: derivationPath,
            extendedPublicKey: extendedPublicKey
        )
    }
}
