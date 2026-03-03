// Copyright (c). Gem Wallet. All rights reserved.

import Testing
import Primitives
import PrimitivesTestKit
import WalletConnectorService
import KeystoreTestKit
import struct Gemstone.SignMessage

@testable import WalletConnector

struct SignMessageSceneViewModelTests {

    @Test
    @MainActor
    func walletTextDisplaysPayloadWallet() {
        let wallet = Wallet.mock(name: "My Secure Wallet")
        let payload = SignMessagePayload(
            chain: .ethereum,
            session: .mock(),
            wallet: wallet,
            message: SignMessage(chain: "ethereum", signType: .eip191, data: "test".data(using: .utf8)!)
        )

        let viewModel = SignMessageSceneViewModel(
            keystore: KeystoreMock(),
            payload: payload,
            confirmTransferDelegate: { _ in }
        )

        #expect(viewModel.walletText == "My Secure Wallet")
    }

    @Test
    @MainActor
    func connectionViewModelUsesPayloadWallet() {
        let wallet = Wallet.mock(id: "multicoin_0xspecific", name: "Test Wallet")
        let session = WalletConnectionSession.mock(sessionId: "test-session")
        let payload = SignMessagePayload(
            chain: .ethereum,
            session: session,
            wallet: wallet,
            message: SignMessage(chain: "ethereum", signType: .eip191, data: "test".data(using: .utf8)!)
        )

        let viewModel = SignMessageSceneViewModel(
            keystore: KeystoreMock(),
            payload: payload,
            confirmTransferDelegate: { _ in }
        )

        #expect(viewModel.connectionViewModel.connection.wallet.id == "multicoin_0xspecific")
        #expect(viewModel.connectionViewModel.connection.wallet.name == "Test Wallet")
    }

    @Test
    @MainActor
    func payloadStoresValidatedChainNotMessageChain() {
        let payload = SignMessagePayload(
            chain: .ethereum,
            session: .mock(),
            wallet: .mock(),
            message: SignMessage(chain: "bitcoin", signType: .eip191, data: "test".data(using: .utf8)!)
        )

        #expect(payload.chain == .ethereum)
        #expect(payload.message.chain == "bitcoin")
    }

    @Test
    @MainActor
    func networkTextUsesPayloadChain() {
        let payload = SignMessagePayload(
            chain: .ethereum,
            session: .mock(),
            wallet: .mock(),
            message: SignMessage(chain: "bitcoin", signType: .eip191, data: "test".data(using: .utf8)!)
        )

        let viewModel = SignMessageSceneViewModel(
            keystore: KeystoreMock(),
            payload: payload,
            confirmTransferDelegate: { _ in }
        )

        #expect(viewModel.networkText == "Ethereum")
    }
}
