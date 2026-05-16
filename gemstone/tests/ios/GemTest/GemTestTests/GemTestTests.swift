// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
@testable import GemTest
import Testing

struct GemTestTests {
    @Test
    func testLoadFFI() async throws {
        let result = Gemstone.libVersion()
        #expect(!result.isEmpty)
    }

    @Test
    func testGetExplorerName() throws {
        let chain = "bitcoin" // Primitive::Chain::Bitcion as_str()
        let explorers = Config().getBlockExplorers(chain: chain)

        try #require(explorers.count >= 2)
        #expect(explorers[1] == "Mempool")

        let explorer = Explorer(chain: chain)
        let txUrl = explorer.getTransactionUrl(
            explorerName: explorers[1],
            transactionId:
            "813d80363c09b1c4d3f0c6ce3382a048b320edefb573a8aedbc7ddd4c65cf7e4"
        )

        #expect(txUrl == "https://mempool.space/tx/813d80363c09b1c4d3f0c6ce3382a048b320edefb573a8aedbc7ddd4c65cf7e4")
    }

    @Test
    func testCache() async throws {
        let cache = Cache<AlienTarget, Data>()
        let target = AlienTarget(
            url: "https://example.com",
            method: .get,
            headers: .none,
            body: .none
        )
        let data = try #require(Data(hex: "0xdeadbeef"))

        await cache.set(value: data, forKey: target, ttl: 1)
        let value = await cache.get(key: target)

        #expect(value == data)

        try await Task.sleep(nanoseconds: 1_100_000_000)
        let expiredValue = await cache.get(key: target)

        #expect(expiredValue == nil)
    }

    @Test
    func testMessagePreview() async throws {
        let base58 = try #require("jo91waLQA1NNeBmZKUF".data(using: .utf8))
        let message = SignMessage(chain: "solana", signType: .base58, data: base58)
        let signer = MessageSigner(message: message)
        let preview = try signer.preview()

        switch preview {
        case .text(let text):
            #expect(text == "this is a test")
        case .eip712, .siwe:
            Issue.record("Unexpected result")
        }

        let result = signer.getResult(
            data: try #require(Data(hex: "7468697320697320612074657374"))
        )
        #expect(result == "jo91waLQA1NNeBmZKUF")
    }

    @Test
    func testMessageHash() async throws {
        let message = SignMessage(
            chain: "ethereum",
            signType: .eip191,
            data: "hello world".data(using: .utf8)!
        )
        let signer = MessageSigner(message: message)
        let hash = try signer.hash()

        #expect(hash.hexString() == "d9eba16ed0ecae432b71fe008c98cc872bb4cc214d3220a36f365326cf807d68")
    }

    @Test
    func testEthereumCallDecoder() throws {
        let decoder = EthereumDecoder()

        // Test ERC-20 transfer without ABI (should auto-detect)
        let erc20Transfer = "0xa9059cbb00000000000000000000000095222290dd7278aa3ddd389cc1e1d165cc4bafe50000000000000000000000000000000000000000000000000de0b6b3a7640000"
        let erc20Result = try decoder.decodeCall(calldata: erc20Transfer, abi: nil)

        #expect(erc20Result.function == "transfer")
        #expect(erc20Result.params.count == 2)
        #expect(erc20Result.params[0].name == "to")
        #expect(erc20Result.params[0].type == "address")
        #expect(erc20Result.params[0].value == "0x95222290DD7278Aa3Ddd389Cc1E1d165CC4BAfe5")
        #expect(erc20Result.params[1].name == "value")
        #expect(erc20Result.params[1].type == "uint256")
        #expect(erc20Result.params[1].value == "1000000000000000000")

        // Test ERC-721 safeTransferFrom with custom ABI
        let erc721Transfer = "0x42842e0e0000000000000000000000008ba1f109551bd432803012645aac136c0c3def25000000000000000000000000271682deb8c4e0901d1a1550ad2e64d568e69909000000000000000000000000000000000000000000000000000000000000007b"
        let erc721ABI = """
        [{
            "inputs": [
                {"name": "from", "type": "address"},
                {"name": "to", "type": "address"},
                {"name": "tokenId", "type": "uint256"}
            ],
            "name": "safeTransferFrom",
            "outputs": [],
            "stateMutability": "nonpayable",
            "type": "function"
        }]
        """

        let erc721Result = try decoder.decodeCall(calldata: erc721Transfer, abi: erc721ABI)

        #expect(erc721Result.function == "safeTransferFrom")
        #expect(erc721Result.params.count == 3)
        #expect(erc721Result.params[0].name == "from")
        #expect(erc721Result.params[0].type == "address")
        #expect(erc721Result.params[0].value == "0x8Ba1f109551bd432803012645aAC136C0c3Def25")
        #expect(erc721Result.params[1].name == "to")
        #expect(erc721Result.params[1].type == "address")
        #expect(erc721Result.params[1].value == "0x271682DEB8C4E0901D1a1550aD2e64D568E69909")
        #expect(erc721Result.params[2].name == "tokenId")
        #expect(erc721Result.params[2].type == "uint256")
        #expect(erc721Result.params[2].value == "123")
    }
}
