import Foundation
import Testing
import Primitives
import PrimitivesTestKit
import WalletConnectorService
import struct Gemstone.SignMessage

@testable import WalletConnector

struct WalletConnectorPresenterTests {

    @Test
    @MainActor
    func completeDismissesSignMessageSheet() {
        let presenter = WalletConnectorPresenter()
        let type = WalletConnectorSheetType.signMessage(
            TransferDataCallback(
                payload: SignMessagePayload(
                    chain: .ethereum,
                    session: .mock(),
                    wallet: .mock(),
                    message: SignMessage(chain: "ethereum", signType: .eip191, data: Data("test".utf8)),
                    simulation: .mock()
                ),
                delegate: { _ in }
            )
        )

        presenter.isPresentingSheet = type
        presenter.complete(type: type)

        #expect(presenter.isPresentingSheet == nil)
    }

    @Test
    @MainActor
    func cancelSheetRejectsSignMessageRequest() async {
        let presenter = WalletConnectorPresenter()
        let manager = WalletConnectorManager(presenter: presenter)
        let payload = SignMessagePayload(
            chain: .ethereum,
            session: .mock(),
            wallet: .mock(),
            message: SignMessage(chain: "ethereum", signType: .eip191, data: Data("test".utf8)),
            simulation: .mock()
        )

        let task = Task {
            try await manager.signMessage(payload: payload)
        }

        guard let type = await presentedSheet(from: presenter) else {
            Issue.record("Expected sign message sheet to be presented")
            return
        }

        await MainActor.run {
            presenter.cancelSheet(type: type)
        }

        await #expect(throws: ConnectionsError.userCancelled) {
            try await task.value
        }
        await MainActor.run {
            #expect(presenter.isPresentingSheet == nil)
        }
    }
}


private func presentedSheet(from presenter: WalletConnectorPresenter) async -> WalletConnectorSheetType? {
    for _ in 0..<10 {
        if let sheet = await MainActor.run(body: { presenter.isPresentingSheet }) {
            return sheet
        }
        await Task.yield()
    }
    return nil
}
