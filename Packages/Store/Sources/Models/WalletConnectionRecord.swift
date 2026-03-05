// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import GRDB

struct WalletConnectionRecord: Codable, FetchableRecord, PersistableRecord  {
    static let databaseTableName: String = "wallets_connections"
    
    enum Columns {
        static let id = Column("id")
        static let sessionId = Column("sessionId")
        static let walletId = Column("walletId")
        static let state = Column("state")
        static let chains = Column("chains")
        static let createdAt = Column("createdAt")
        static let expireAt = Column("expireAt")
        static let appName = Column("appName")
        static let appDescription = Column("appDescription")
        static let appLink = Column("appLink")
        static let appIcon = Column("appIcon")
    }

    var id: String
    var sessionId: String
    var walletId: String
    var state: WalletConnectionState
    var chains: [Chain]?
    var createdAt: Date
    var expireAt: Date
    
    // metadata
    var appName: String
    var appDescription: String
    var appLink: String
    var appIcon: String
}

extension WalletConnectionRecord: CreateTable {
    static func create(db: Database) throws {
        try db.create(table: Self.databaseTableName, ifNotExists: true) {
            $0.column(Columns.id.name, .text)
                .primaryKey()
                .notNull()
            $0.column(Columns.sessionId.name, .text)
                .notNull()
            $0.column(Columns.walletId.name, .text)
                .notNull()
                .indexed()
                .references(WalletRecord.databaseTableName, onDelete: .cascade, onUpdate: .cascade)
            $0.column(Columns.state.name, .text)
                .notNull()
            $0.column(Columns.chains.name, .jsonText)
            $0.column(Columns.createdAt.name, .date)
                .notNull()
            $0.column(Columns.expireAt.name, .date)
                .notNull()
            
            $0.column(Columns.appName.name, .text)
                .notNull()
            $0.column(Columns.appDescription.name, .text)
                .notNull()
            $0.column(Columns.appLink.name, .text)
                .notNull()
            $0.column(Columns.appIcon.name, .text)
                .notNull()
        }
    }
}

extension WalletConnection {
    var record: WalletConnectionRecord {
        WalletConnectionRecord(
            id: session.sessionId,
            sessionId: session.sessionId,
            walletId: wallet.id,
            state: session.state,
            chains: session.chains,
            createdAt: Date(),
            expireAt: session.expireAt,
            appName: session.metadata.name,
            appDescription: session.metadata.description,
            appLink: session.metadata.url,
            appIcon: session.metadata.icon
        )
    }
}

extension WalletConnectionRecord {
    var session: WalletConnectionSession {
        WalletConnectionSession(
            id: sessionId,
            sessionId: sessionId,
            state: state,
            chains: chains,
            createdAt: createdAt,
            expireAt: expireAt,
            metadata: WalletConnectionSessionAppMetadata(
                name: appName,
                description: appDescription,
                url: appLink,
                icon: appIcon
            )
        )
    }
    
    func update(with session: WalletConnectionSession) -> WalletConnectionRecord {
        WalletConnectionRecord(
            id: session.sessionId,
            sessionId: session.sessionId,
            walletId: walletId,
            state: session.state,
            chains: chains ?? [],
            createdAt: createdAt,
            expireAt: session.expireAt,
            appName: session.metadata.name,
            appDescription: session.metadata.description,
            appLink: session.metadata.url,
            appIcon: session.metadata.icon
        )
    }
}
