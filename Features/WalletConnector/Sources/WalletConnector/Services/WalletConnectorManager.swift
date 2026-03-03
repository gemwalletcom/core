// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI
import Primitives
import WalletConnectorService

public final class WalletConnectorManager {
    public let presenter: WalletConnectorPresenter

    public init(presenter: WalletConnectorPresenter) {
        self.presenter = presenter
    }
}

// MARK: - WalletConnectorInteractable

extension WalletConnectorManager: WalletConnectorInteractable {
    public func sessionReject(error: any Error) async {
        let ignoreErrors = [
            "User cancelled" // User cancelled throw by WalletConnect if session proposal is rejected
        ]
        guard !ignoreErrors.contains(error.localizedDescription) else {
            return
        }
        await MainActor.run { [weak self] in
            guard let self else { return }
            self.presenter.isPresentingError = error.localizedDescription
        }
    }

    public func sessionApproval(payload: WCPairingProposal) async throws -> WalletId {
        let value = try await presentSheet(payload: payload, sheetType: { .connectionProposal($0) })
        return try WalletId.from(id: value)
    }

    public func signMessage(payload: SignMessagePayload) async throws -> String {
        try await presentSheet(payload: payload, sheetType: { .signMessage($0) })
    }

    public func sendTransaction(transferData: WCTransferData) async throws -> String {
        try await presentSheet(payload: transferData, sheetType: { .transferData($0) })
    }

    public func signTransaction(transferData: WCTransferData) async throws -> String {
        try await presentSheet(payload: transferData, sheetType: { .transferData($0) })
    }

    public func sendRawTransaction(transferData: WCTransferData) async throws -> String {
        fatalError("")
    }

    // MARK: - Private

    private func presentSheet<T: Identifiable & Sendable>(
        payload: T,
        sheetType: @Sendable @escaping (TransferDataCallback<T>) -> WalletConnectorSheetType
    ) async throws -> String {
        let (stream, continuation) = AsyncThrowingStream.makeStream(of: String.self)

        let callback = TransferDataCallback(payload: payload) {
            continuation.yield(with: $0)
            continuation.finish()
        }

        await MainActor.run { [weak self] in
            self?.presenter.isPresentingSheet = sheetType(callback)
        }

        for try await value in stream {
            return value
        }
        throw ConnectionsError.userCancelled
    }
}
