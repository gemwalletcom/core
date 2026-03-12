// Copyright (c). Gem Wallet. All rights reserved.

import Testing
import BigInt
import Foundation
import Primitives
import PrimitivesComponents
import PrimitivesTestKit
import WalletConnectorService
import WalletConnectorServiceTestKit
import AddressNameServiceTestKit
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
            message: SignMessage(chain: "ethereum", signType: .eip191, data: "test".data(using: .utf8)!),
            simulation: .mock()
        )

        let viewModel = SignMessageSceneViewModel(
            keystore: KeystoreMock(),
            addressNameService: .mock(),
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
            message: SignMessage(chain: "ethereum", signType: .eip191, data: "test".data(using: .utf8)!),
            simulation: .mock()
        )

        let viewModel = SignMessageSceneViewModel(
            keystore: KeystoreMock(),
            addressNameService: .mock(),
            payload: payload,
            confirmTransferDelegate: { _ in }
        )

        #expect(viewModel.connectionViewModel.connection.wallet.id == "multicoin_0xspecific")
        #expect(viewModel.connectionViewModel.connection.wallet.name == "Test Wallet")
    }

    @Test
    @MainActor
    func appTextUsesShortNameWithoutDomain() {
        let payload = SignMessagePayload.mock(
            session: .mock(metadata: .mock(
                name: "PancakeSwap - Trade",
                url: "https://pancakeswap.finance/swap"
            ))
        )

        let viewModel = SignMessageSceneViewModel(
            keystore: KeystoreMock(),
            addressNameService: .mock(),
            payload: payload,
            confirmTransferDelegate: { _ in }
        )

        #expect(viewModel.appText == "PancakeSwap")
    }

    @Test
    @MainActor
    func titleUsesReviewRequest() {
        let payload = SignMessagePayload(
            chain: .ethereum,
            session: .mock(),
            wallet: .mock(),
            message: SignMessage(chain: "ethereum", signType: .eip191, data: "test".data(using: .utf8)!),
            simulation: .mock()
        )

        let viewModel = SignMessageSceneViewModel(
            keystore: KeystoreMock(),
            addressNameService: .mock(),
            payload: payload,
            confirmTransferDelegate: { _ in }
        )

        #expect(viewModel.title == "Review Request")
    }

    @Test
    @MainActor
    func payloadStoresValidatedChainNotMessageChain() {
        let payload = SignMessagePayload(
            chain: .ethereum,
            session: .mock(),
            wallet: .mock(),
            message: SignMessage(chain: "bitcoin", signType: .eip191, data: "test".data(using: .utf8)!),
            simulation: .mock()
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
            message: SignMessage(chain: "bitcoin", signType: .eip191, data: "test".data(using: .utf8)!),
            simulation: .mock()
        )

        let viewModel = SignMessageSceneViewModel(
            keystore: KeystoreMock(),
            addressNameService: .mock(),
            payload: payload,
            confirmTransferDelegate: { _ in }
        )

        #expect(viewModel.networkText == "Ethereum")
    }

    @Test
    @MainActor
    func contextRowsProvideWalletAndNetworkImages() {
        let payload = SignMessagePayload(
            chain: .ethereum,
            session: .mock(),
            wallet: .mock(),
            message: SignMessage(chain: "ethereum", signType: .eip191, data: "test".data(using: .utf8)!),
            simulation: .mock()
        )

        let viewModel = SignMessageSceneViewModel(
            keystore: KeystoreMock(),
            addressNameService: .mock(),
            payload: payload,
            confirmTransferDelegate: { _ in }
        )

        #expect(viewModel.walletAssetImage == WalletViewModel(wallet: payload.wallet).avatarImage)
        #expect(viewModel.networkAssetImage == AssetIdViewModel(assetId: payload.chain.asset.id).networkAssetImage)
    }

    @Test
    @MainActor
    func buttonEnabledWithNoWarnings() {
        let payload = SignMessagePayload(
            chain: .ethereum,
            session: .mock(),
            wallet: .mock(),
            message: SignMessage(chain: "ethereum", signType: .eip191, data: "test".data(using: .utf8)!),
            simulation: .mock()
        )

        let viewModel = SignMessageSceneViewModel(
            keystore: KeystoreMock(),
            addressNameService: .mock(),
            payload: payload,
            confirmTransferDelegate: { _ in }
        )

        #expect(!viewModel.isButtonDisabled)
    }

    @Test
    @MainActor
    func buttonEnabledWithNonCriticalWarnings() {
        let payload = SignMessagePayload(
            chain: .ethereum,
            session: .mock(),
            wallet: .mock(),
            message: SignMessage(chain: "ethereum", signType: .eip191, data: "test".data(using: .utf8)!),
            simulation: .mock(warnings: [SimulationWarning(
                severity: .warning,
                warning: .tokenApproval(assetId: AssetId(chain: .ethereum, tokenId: "0x123"), value: nil),
                message: nil
            )])
        )

        let viewModel = SignMessageSceneViewModel(
            keystore: KeystoreMock(),
            addressNameService: .mock(),
            payload: payload,
            confirmTransferDelegate: { _ in }
        )

        #expect(!viewModel.isButtonDisabled)
    }

    @Test
    @MainActor
    func simulationWarningsPassThroughUnlimitedAndFiniteApprovals() {
        let payload = SignMessagePayload(
            chain: .ethereum,
            session: .mock(),
            wallet: .mock(),
            message: SignMessage(chain: "ethereum", signType: .eip191, data: "test".data(using: .utf8)!),
            simulation: .mock(warnings: [
                SimulationWarning(
                    severity: .warning,
                    warning: .tokenApproval(assetId: AssetId(chain: .ethereum, tokenId: "0x123"), value: BigInt(1000)),
                    message: nil
                ),
                SimulationWarning(
                    severity: .warning,
                    warning: .tokenApproval(assetId: AssetId(chain: .ethereum, tokenId: "0x123"), value: nil),
                    message: nil
                ),
            ])
        )

        let viewModel = SignMessageSceneViewModel(
            keystore: KeystoreMock(),
            addressNameService: .mock(),
            payload: payload,
            confirmTransferDelegate: { _ in }
        )

        #expect(viewModel.simulationWarnings.count == 2)
        #expect(viewModel.simulationWarnings.last?.warning == .tokenApproval(assetId: AssetId(chain: .ethereum, tokenId: "0x123"), value: nil))
    }

    @Test
    @MainActor
    func buttonDisabledWithCriticalWarnings() {
        let payload = SignMessagePayload(
            chain: .ethereum,
            session: .mock(),
            wallet: .mock(),
            message: SignMessage(chain: "ethereum", signType: .eip191, data: "test".data(using: .utf8)!),
            simulation: .mock(warnings: [SimulationWarning(severity: .critical, warning: .suspiciousSpender, message: nil)])
        )

        let viewModel = SignMessageSceneViewModel(
            keystore: KeystoreMock(),
            addressNameService: .mock(),
            payload: payload,
            confirmTransferDelegate: { _ in }
        )

        #expect(viewModel.isButtonDisabled)
    }

    @Test
    @MainActor
    func simulationWarningsPassThroughExternallyOwnedSpenderWarnings() {
        let payload = SignMessagePayload(
            chain: .ethereum,
            session: .mock(),
            wallet: .mock(),
            message: SignMessage(chain: "ethereum", signType: .eip191, data: "test".data(using: .utf8)!),
            simulation: .mock(warnings: [
                SimulationWarning(
                    severity: .warning,
                    warning: .permitApproval(assetId: AssetId(chain: .ethereum, tokenId: "0x123"), value: BigInt(1000)),
                    message: nil
                ),
                SimulationWarning(
                    severity: .critical,
                    warning: .externallyOwnedSpender,
                    message: nil
                ),
            ])
        )

        let viewModel = SignMessageSceneViewModel(
            keystore: KeystoreMock(),
            addressNameService: .mock(),
            payload: payload,
            confirmTransferDelegate: { _ in }
        )

        #expect(viewModel.simulationWarnings.count == 2)
        #expect(viewModel.simulationWarnings.last?.warning == .externallyOwnedSpender)
    }

    @Test
    @MainActor
    func permitBatchExternallyOwnedSpenderKeepsWarningAndPayload() {
        let message = """
        {
          "types": {
            "EIP712Domain": [
              { "name": "name", "type": "string" },
              { "name": "chainId", "type": "uint256" },
              { "name": "verifyingContract", "type": "address" }
            ],
            "PermitBatch": [
              { "name": "details", "type": "PermitDetails[]" },
              { "name": "spender", "type": "address" },
              { "name": "sigDeadline", "type": "uint256" }
            ],
            "PermitDetails": [
              { "name": "token", "type": "address" },
              { "name": "amount", "type": "uint160" },
              { "name": "expiration", "type": "uint48" },
              { "name": "nonce", "type": "uint48" }
            ]
          },
          "primaryType": "PermitBatch",
          "domain": {
            "name": "Permit2",
            "chainId": "1",
            "verifyingContract": "0x000000000022D473030F116dDEE9F6B43aC78BA3"
          },
          "message": {
            "details": [
              {
                "token": "0x1111111111111111111111111111111111111111",
                "amount": "1000000000000000000",
                "expiration": "1712600000",
                "nonce": "0"
              }
            ],
            "spender": "0x3333333333333333333333333333333333333333",
            "sigDeadline": "1712600500"
          }
        }
        """
        let payload = SignMessagePayload(
            chain: .ethereum,
            session: .mock(),
            wallet: .mock(),
            message: SignMessage(chain: "ethereum", signType: .eip712, data: Data(message.utf8)),
            simulation: .mock(
                warnings: [SimulationWarning(severity: .critical, warning: .externallyOwnedSpender, message: nil)],
                payload: [
                    .standard(kind: .contract, value: "0x000000000022D473030F116dDEE9F6B43aC78BA3", fieldType: .address, display: .primary),
                    .standard(kind: .method, value: "Permit Batch", fieldType: .text, display: .primary),
                    .standard(kind: .spender, value: "0x3333333333333333333333333333333333333333", fieldType: .address, display: .primary),
                ]
            )
        )

        let viewModel = SignMessageSceneViewModel(
            keystore: KeystoreMock(),
            addressNameService: .mock(),
            payload: payload,
            confirmTransferDelegate: { _ in }
        )

        #expect(viewModel.simulationWarnings.count == 1)
        #expect(viewModel.simulationWarnings.first?.warning == .externallyOwnedSpender)
        #expect(viewModel.isButtonDisabled)
        #expect(viewModel.hasPayload)
        #expect(viewModel.primaryPayloadFields.contains(where: { $0.kind == .spender && $0.value == "0x3333333333333333333333333333333333333333" }))
    }

    @Test
    @MainActor
    func simulationWarningsPassThroughValidationWarnings() {
        let payload = SignMessagePayload(
            chain: .ethereum,
            session: .mock(),
            wallet: .mock(),
            message: SignMessage(chain: "ethereum", signType: .eip191, data: "test".data(using: .utf8)!),
            simulation: .mock(warnings: [
                SimulationWarning(
                    severity: .warning,
                    warning: .permitApproval(assetId: AssetId(chain: .ethereum, tokenId: "0x123"), value: BigInt(1000)),
                    message: nil
                ),
                SimulationWarning(
                    severity: .critical,
                    warning: .validationError,
                    message: "Unable to verify spender is a contract"
                ),
            ])
        )

        let viewModel = SignMessageSceneViewModel(
            keystore: KeystoreMock(),
            addressNameService: .mock(),
            payload: payload,
            confirmTransferDelegate: { _ in }
        )

        #expect(viewModel.simulationWarnings.count == 2)
        #expect(viewModel.simulationWarnings.last?.warning == .validationError)
    }

    @Test
    @MainActor
    func siweChainMismatchStillUsesStructuredPayload() {
        let message = [
            "thepoc.xyz wants you to sign in with your Ethereum account:",
            "0xBA4D1d35bCe0e8F28E5a3403e7a0b996c5d50AC4",
            "",
            "Sign in with different chain ID",
            "",
            "URI: https://thepoc.xyz",
            "Version: 1",
            "Chain ID: 137",
            "Nonce: gv7zples2q60kq7bnamtuwo",
            "Issued At: 2026-03-11T04:20:21.742Z",
        ]
        .joined(separator: "\n")

        let payload = SignMessagePayload(
            chain: .ethereum,
            session: .mock(),
            wallet: .mock(),
            message: SignMessage(chain: "ethereum", signType: .eip191, data: Data(message.utf8)),
            simulation: .mock(warnings: [
                SimulationWarning(severity: .critical, warning: .validationError, message: "Chain ID mismatch"),
            ])
        )

        let viewModel = SignMessageSceneViewModel(
            keystore: KeystoreMock(),
            addressNameService: .mock(),
            payload: payload,
            confirmTransferDelegate: { _ in }
        )

        #expect(viewModel.hasPayload)
        #expect(viewModel.primaryPayloadFields.count == 2)
    }

}
