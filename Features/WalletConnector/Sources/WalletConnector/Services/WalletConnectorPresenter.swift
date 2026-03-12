// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI

@Observable
public final class WalletConnectorPresenter: Sendable {

    @MainActor
    public var isPresentingError: String?
    @MainActor
    public var isPresentingConnectionBar: Bool = false
    @MainActor
    public var isPresentingSheet: WalletConnectorSheetType?

    public init() { }

    @MainActor
    public func complete(type: WalletConnectorSheetType) {
        guard isPresentingSheet?.id == type.id else {
            return
        }
        isPresentingSheet = nil
    }

    @MainActor
    public func cancelSheet(type: WalletConnectorSheetType) {
        guard isPresentingSheet?.id == type.id else {
            return
        }

        type.reject(ConnectionsError.userCancelled)
        self.isPresentingSheet = nil
    }
}
