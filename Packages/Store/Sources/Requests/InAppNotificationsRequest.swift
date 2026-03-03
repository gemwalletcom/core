// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GRDB
import Primitives

public struct InAppNotificationsRequest: DatabaseQueryable {

    public var walletId: String

    public init(walletId: String) {
        self.walletId = walletId
    }

    public func fetch(_ db: Database) throws -> [Primitives.InAppNotification] {
        try NotificationRecord
            .filter(NotificationRecord.Columns.walletId == walletId)
            .order(NotificationRecord.Columns.createdAt.desc)
            .fetchAll(db)
            .map { try $0.mapToNotification() }
    }
}

extension InAppNotificationsRequest: Equatable {}
