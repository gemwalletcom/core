// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import Primitives
import PrimitivesTestKit
@testable import Signer
import Testing

private let swapTestPrivateKey = Data(repeating: 0x11, count: 32)

private enum TestValues {
    static let ethereumSender = "0x1111111111111111111111111111111111111111"
    static let ethereumReceiver = "0x2222222222222222222222222222222222222222"
    static let ethereumAggregator = "0x3333333333333333333333333333333333333333"
    static let nearSender = "sender.near"
    static let nearReceiver = "receiver.near"
    static let suiSender = "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
    static let suiReceiver = "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
    static let tronSender = "TMwFHYXLJaRUPeW6421aqXL4ZEzPRFGkGT"
    static let tronAggregator = "TJApZYJwPKuQR7tL6FmvD6jDjbYpHESZGH"
    static let tronDestinationHex = "4199066fd9daa7a14e000f63b8803138607dc00aaa"
    static let tronDestination = "TPvL6et9hcRMb3j9vzVQRtt4UC2HvQrmCK"
    static let tronTokenId = "TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t"
}

struct SwapSignerTests {
    private func makeSwapInput(
        from fromAsset: Asset,
        to toAsset: Asset,
        swapData: SwapData,
        value: BigInt,
        metadata: TransactionLoadMetadata = .none,
        useMaxAmount: Bool,
        senderAddress: String? = nil,
        destinationAddress: String? = nil
    ) -> SignerInput {
        SignerInput(
            type: .swap(fromAsset, toAsset, swapData),
            asset: fromAsset,
            value: value,
            fee: .mock(),
            isMaxAmount: useMaxAmount,
            memo: nil,
            senderAddress: senderAddress ?? swapData.quote.fromAddress,
            destinationAddress: destinationAddress ?? swapData.quote.toAddress,
            metadata: metadata
        )
    }

    private func makeSwapData(
        walletAddress: String,
        toAddress: String,
        destinationAddress: String,
        data: String,
        fromValue: String = "1000",
        useMaxAmount: Bool = false
    ) -> SwapData {
        SwapData(
            quote: SwapQuote(
                fromAddress: walletAddress,
                fromValue: fromValue,
                toAddress: destinationAddress,
                toValue: "2000",
                providerData: SwapProviderData(
                    provider: .nearIntents,
                    name: "Near Intents",
                    protocolName: "near_intents"
                ),
                slippageBps: 50,
                etaInSeconds: 60,
                useMaxAmount: useMaxAmount
            ),
            data: SwapQuoteData(
                to: toAddress,
                dataType: .transfer,
                value: "0",
                data: data,
                memo: nil,
                approval: nil,
                gasLimit: nil
            )
        )
    }

    @Test
    func nativeMaxAmountUsesFeeAdjustedValue() throws {
        let fromAsset = Asset.mockEthereum()
        let toAsset = Asset.mockNear()
        let feeAdjustedValue: BigInt = 9500
        let swapData = makeSwapData(
            walletAddress: TestValues.ethereumSender,
            toAddress: TestValues.ethereumReceiver,
            destinationAddress: TestValues.ethereumReceiver,
            data: "0x",
            fromValue: "9000"
        )
        let input = makeSwapInput(
            from: fromAsset,
            to: toAsset,
            swapData: swapData,
            value: feeAdjustedValue,
            useMaxAmount: true,
            senderAddress: TestValues.ethereumSender
        )
        let mockSigner = SwapSignableMock()
        let swapSigner = SwapSigner()

        let result = try swapSigner.signSwap(
            signer: mockSigner,
            input: input,
            fromAsset: fromAsset,
            swapData: swapData,
            privateKey: swapTestPrivateKey
        )

        #expect(result == [mockSigner.transferResult])
        #expect(mockSigner.transferInputs.count == 1)
        #expect(mockSigner.tokenTransferInputs.isEmpty)

        let transferInput = mockSigner.transferInputs.first!
        #expect(transferInput.asset == fromAsset)
        if case .transfer(let asset) = transferInput.type {
            #expect(asset == fromAsset)
        } else {
            #expect(Bool(false))
        }
        #expect(transferInput.destinationAddress == swapData.data.to)
        #expect(transferInput.useMaxAmount == true)
        #expect(transferInput.value == feeAdjustedValue)
        #expect(transferInput.value != swapData.quote.fromValueBigInt)
    }

    @Test
    func nativeNonMaxAmountUsesQuoteValue() throws {
        let fromAsset = Asset.mockEthereum()
        let toAsset = Asset.mockNear()
        let swapData = makeSwapData(
            walletAddress: TestValues.ethereumSender,
            toAddress: TestValues.ethereumReceiver,
            destinationAddress: TestValues.ethereumReceiver,
            data: "0x",
            fromValue: "9000"
        )
        let input = makeSwapInput(
            from: fromAsset,
            to: toAsset,
            swapData: swapData,
            value: 9500,
            useMaxAmount: false,
            senderAddress: TestValues.ethereumSender
        )
        let mockSigner = SwapSignableMock()

        _ = try SwapSigner().signSwap(
            signer: mockSigner,
            input: input,
            fromAsset: fromAsset,
            swapData: swapData,
            privateKey: swapTestPrivateKey
        )

        let transferInput = mockSigner.transferInputs.first!
        #expect(transferInput.value == swapData.quote.fromValueBigInt)
    }

    @Test
    func tokenMaxAmountUsesQuoteValue() throws {
        let fromAsset = Asset.mockEthereumUSDT()
        let toAsset = Asset.mockNear()
        let swapData = makeSwapData(
            walletAddress: TestValues.ethereumSender,
            toAddress: TestValues.ethereumReceiver,
            destinationAddress: TestValues.ethereumReceiver,
            data: "0x",
            fromValue: "9000"
        )
        let input = makeSwapInput(
            from: fromAsset,
            to: toAsset,
            swapData: swapData,
            value: 9500,
            useMaxAmount: true,
            senderAddress: TestValues.ethereumSender
        )
        let mockSigner = SwapSignableMock()

        _ = try SwapSigner().signSwap(
            signer: mockSigner,
            input: input,
            fromAsset: fromAsset,
            swapData: swapData,
            privateKey: swapTestPrivateKey
        )

        let transferInput = mockSigner.tokenTransferInputs.first!
        #expect(transferInput.value == swapData.quote.fromValueBigInt)
    }

    @Test
    func nearTransferSwapKeepsMetadataAndUsesTransfer() throws {
        let fromAsset = Asset.mockNear()
        let toAsset = Asset.mockEthereum()
        let swapData = makeSwapData(
            walletAddress: TestValues.nearSender,
            toAddress: TestValues.nearReceiver,
            destinationAddress: TestValues.nearReceiver,
            data: "0x"
        )
        let metadata: TransactionLoadMetadata = .near(
            sequence: 42,
            blockHash: "near-block-hash"
        )
        let input = makeSwapInput(
            from: fromAsset,
            to: toAsset,
            swapData: swapData,
            value: swapData.quote.fromValueBigInt,
            metadata: metadata,
            useMaxAmount: false,
            senderAddress: TestValues.nearSender,
            destinationAddress: TestValues.nearReceiver
        )
        let mockSigner = SwapSignableMock()
        let swapSigner = SwapSigner()

        let result = try swapSigner.signSwap(
            signer: mockSigner,
            input: input,
            fromAsset: fromAsset,
            swapData: swapData,
            privateKey: swapTestPrivateKey
        )

        #expect(result == [mockSigner.transferResult])
        #expect(mockSigner.transferInputs.count == 1)
        #expect(mockSigner.tokenTransferInputs.isEmpty)

        guard let captured = mockSigner.transferInputs.first else {
            #expect(Bool(false))
            return
        }

        #expect(captured.destinationAddress == swapData.data.to)
        #expect(captured.value == swapData.quote.fromValueBigInt)

        if case .near(let sequence, let blockHash) = captured.metadata {
            #expect(sequence == 42)
            #expect(blockHash == "near-block-hash")
        } else {
            #expect(Bool(false))
        }
    }

    @Test
    func suiTransferSwapUsesTransferFlow() throws {
        let fromAsset = Asset.mockSUI()
        let toAsset = Asset.mockEthereum()
        let swapData = makeSwapData(
            walletAddress: TestValues.suiSender,
            toAddress: TestValues.suiReceiver,
            destinationAddress: TestValues.suiReceiver,
            data: "0x"
        )
        let metadata: TransactionLoadMetadata = .sui(messageBytes: "payload")
        let input = makeSwapInput(
            from: fromAsset,
            to: toAsset,
            swapData: swapData,
            value: swapData.quote.fromValueBigInt,
            metadata: metadata,
            useMaxAmount: false,
            senderAddress: TestValues.suiSender,
            destinationAddress: TestValues.suiReceiver
        )
        let mockSigner = SwapSignableMock()
        let swapSigner = SwapSigner()

        let result = try swapSigner.signSwap(
            signer: mockSigner,
            input: input,
            fromAsset: fromAsset,
            swapData: swapData,
            privateKey: swapTestPrivateKey
        )

        #expect(result == [mockSigner.transferResult])
        #expect(mockSigner.transferInputs.count == 1)
        #expect(mockSigner.tokenTransferInputs.isEmpty)

        guard let captured = mockSigner.transferInputs.first else {
            #expect(Bool(false))
            return
        }

        #expect(captured.destinationAddress == swapData.data.to)
        #expect(captured.value == swapData.quote.fromValueBigInt)

        if case .sui(let messageBytes) = captured.metadata {
            #expect(messageBytes == "payload")
        } else {
            #expect(Bool(false))
        }
    }
}

private final class SwapSignableMock: Signable {
    var transferInputs: [SignerInput] = []
    var tokenTransferInputs: [SignerInput] = []
    let transferResult: String
    let tokenTransferResult: String

    init(
        transferResult: String = "transfer-signature",
        tokenTransferResult: String = "token-transfer-signature"
    ) {
        self.transferResult = transferResult
        self.tokenTransferResult = tokenTransferResult
    }

    func signTransfer(input: SignerInput, privateKey: Data) throws -> String {
        transferInputs.append(input)
        return transferResult
    }

    func signTokenTransfer(input: SignerInput, privateKey: Data) throws -> String {
        tokenTransferInputs.append(input)
        return tokenTransferResult
    }
}
