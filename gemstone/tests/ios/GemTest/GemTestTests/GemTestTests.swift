// Copyright (c). Gem Wallet. All rights reserved.

import Gemstone
@testable import GemTest
import XCTest

final class GemTestTests: XCTestCase {
    func testLoadFFI() async throws {
        let result = Gemstone.libVersion()
        XCTAssertFalse(result.isEmpty)
    }

    func testGetExplorerName() {
        let chain = "bitcoin" // Primitive::Chain::Bitcion as_str()
        let explorers = Config().getBlockExplorers(chain: chain)

        XCTAssertTrue(explorers.count >= 2)
        XCTAssertEqual(explorers[1], "Mempool")

        let explorer = Explorer(chain: chain)
        let txUrl = explorer.getTransactionUrl(
            explorerName: explorers[1],
            transactionId:
            "813d80363c09b1c4d3f0c6ce3382a048b320edefb573a8aedbc7ddd4c65cf7e4"
        )

        XCTAssertEqual(
            txUrl,
            "https://mempool.space/tx/813d80363c09b1c4d3f0c6ce3382a048b320edefb573a8aedbc7ddd4c65cf7e4"
        )
    }

    func testGetEvmChainConfig() throws {
        let config = Config().getEvmChainConfig(chain: "zksync")

        XCTAssertFalse(config.isOpstack)
    }

    func testCache() async throws {
        let cache = Cache<AlienTarget, Data>()
        let target = AlienTarget(
            url: "https://example.com",
            method: .get,
            headers: .none,
            body: .none
        )
        let data = Data(hex: "0xdeadbeef")!

        await cache.set(value: data, forKey: target, ttl: 1)
        let value = await cache.get(key: target)

        XCTAssertEqual(value, data)

        sleep(1)
        let expiredValue = await cache.get(key: target)

        XCTAssertNil(expiredValue)
    }

    func testMessagePreview() async throws {
        let base58 = "jo91waLQA1NNeBmZKUF".data(using: .utf8)!
        let message = SignMessage(chain: "solana", signType: .base58, data: base58)
        let decoder = SignMessageDecoder(message: message)
        let preview = try decoder.preview()

        switch preview {
        case .text(let text):
            XCTAssertEqual(text, "this is a test")
        case .eip712:
            XCTFail("Unexpected result")
        }

        let result = decoder.getResult(
            data: Data(hex: "7468697320697320612074657374")!
        )
        XCTAssertEqual(result, "jo91waLQA1NNeBmZKUF")
    }

    func testMessageHash() async throws {
        let message = SignMessage(
            chain: "ethereum",
            signType: .eip191,
            data: "hello world".data(using: .utf8)!
        )
        let decoder = SignMessageDecoder(message: message)
        let hash = decoder.hash()

        XCTAssertEqual(
            hash.hexString(),
            "d9eba16ed0ecae432b71fe008c98cc872bb4cc214d3220a36f365326cf807d68"
        )
    }

    func testEthereumCallDecoder() throws {
        let decoder = EthereumDecoder()

        // Test ERC-20 transfer without ABI (should auto-detect)
        let erc20Transfer = "0xa9059cbb00000000000000000000000095222290dd7278aa3ddd389cc1e1d165cc4bafe50000000000000000000000000000000000000000000000000de0b6b3a7640000"
        let erc20Result = try decoder.decodeCall(calldata: erc20Transfer, abi: nil)

        XCTAssertEqual(erc20Result.function, "transfer")
        XCTAssertEqual(erc20Result.params.count, 2)
        XCTAssertEqual(erc20Result.params[0].name, "to")
        XCTAssertEqual(erc20Result.params[0].type, "address")
        XCTAssertEqual(erc20Result.params[0].value, "0x95222290DD7278Aa3Ddd389Cc1E1d165CC4BAfe5")
        XCTAssertEqual(erc20Result.params[1].name, "value")
        XCTAssertEqual(erc20Result.params[1].type, "uint256")
        XCTAssertEqual(erc20Result.params[1].value, "1000000000000000000")

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

        XCTAssertEqual(erc721Result.function, "safeTransferFrom")
        XCTAssertEqual(erc721Result.params.count, 3)
        XCTAssertEqual(erc721Result.params[0].name, "from")
        XCTAssertEqual(erc721Result.params[0].type, "address")
        XCTAssertEqual(erc721Result.params[0].value, "0x8Ba1f109551bd432803012645aAC136C0c3Def25")
        XCTAssertEqual(erc721Result.params[1].name, "to")
        XCTAssertEqual(erc721Result.params[1].type, "address")
        XCTAssertEqual(erc721Result.params[1].value, "0x271682DEB8C4E0901D1a1550aD2e64D568E69909")
        XCTAssertEqual(erc721Result.params[2].name, "tokenId")
        XCTAssertEqual(erc721Result.params[2].type, "uint256")
        XCTAssertEqual(erc721Result.params[2].value, "123")
    }
}
