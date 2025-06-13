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

        XCTAssertEqual(explorers.count, 2)
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

    func testSplitStake() throws {
        let input = SuiStakeInput(
            sender:
            "0xe6af80fe1b0b42fcd96762e5c70f5e8dae39f8f0ee0f118cac0d55b74e2927c2",
            validator:
            "0x61953ea72709eed72f4441dd944eec49a11b4acabfc8e04015e89c63be81b6ab",
            stakeAmount: 1_000_000_000,
            gas: SuiGas(
                budget: 20_000_000,
                price: 750
            ),
            coins: [
                SuiCoin(
                    coinType: "0x2::sui::SUI",
                    balance: 2_500_000_000,
                    objectRef: SuiObjectRef(
                        objectId:
                        "0x48a4367b1e0b4ad996375296e4471ae8d8df2576f90636b8e84051d97c1a363a",
                        digest: "9JHLrv8f3G3NFfWwWt54oHjdZYXD8VaNoZYXu3fV8pmB",
                        version: 65_307_031
                    )
                ),
            ]
        )

        let output = try suiEncodeSplitStake(input: input)

        XCTAssertEqual(
            output.txData.hexString(),
            "000003000800ca9a3b0000000001010000000000000000000000000000000000000000000000000000000000000005010000000000000001002061953ea72709eed72f4441dd944eec49a11b4acabfc8e04015e89c63be81b6ab020200010100000000000000000000000000000000000000000000000000000000000000000000030a7375695f73797374656d11726571756573745f6164645f7374616b6500030101000300000000010200e6af80fe1b0b42fcd96762e5c70f5e8dae39f8f0ee0f118cac0d55b74e2927c20148a4367b1e0b4ad996375296e4471ae8d8df2576f90636b8e84051d97c1a363a9781e40300000000207b4ad8fc8964fdfbdf1c41f4c2ba993d607ae8ed85d260c500b8f4c5855ce0b6e6af80fe1b0b42fcd96762e5c70f5e8dae39f8f0ee0f118cac0d55b74e2927c2ee02000000000000002d31010000000000"
        )
        XCTAssertEqual(
            output.hash.hexString(),
            "0437f7744c84f9226be8a09a76e9bedf4ca961f1937a354f72d5352266a72795"
        )
    }

    func testConvertHRP() throws {
        let address = "cosmos1fxygpgus4nd5jmfl5j7fh5y8hyy53z8u95dzx7"

        XCTAssertEqual(
            try cosmosConvertHrp(address: address, hrp: "noble"),
            "noble1fxygpgus4nd5jmfl5j7fh5y8hyy53z8udhc27s"
        )
    }

    func testDecodingBscDelegations() throws {
        let result = Data(
            hex:
            "00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000002000000000000000000000000ee448667ffc3d15ca023a6deef2d0faf084c0716000000000000000000000000773760b0708a5cc369c346993a0c225d8e4043b10000000000000000000000000000000000000000000000000de0b6b3b015a6430000000000000000000000000000000000000000000000000dd62dce1850f388000000000000000000000000ee448667ffc3d15ca023a6deef2d0faf084c0716000000000000000000000000343da7ff0446247ca47aa41e2a25c5bbb230ed0a0000000000000000000000000000000000000000000000000e09ef1d9101a1740000000000000000000000000000000000000000000000000e028d70463b87f8"
        )!

        let delegations = try Gemstone.bscDecodeDelegationsReturn(
            result: result
        )

        XCTAssertEqual(delegations.count, 2)
        XCTAssertEqual(
            delegations[1].validatorAddress,
            "0x343dA7Ff0446247ca47AA41e2A25c5Bbb230ED0A"
        )
        XCTAssertEqual(delegations[1].amount, "1011602501587280244")
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
        let message = SignMessage(signType: .base58, data: base58)
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

    func testPriorityFees() throws {
        let feeHisotry = GemEthereumFeeHistory(
            reward: [
                [
                    "0xaa33c7",
                    "0x54b27f6",
                    "0x17016b10",
                ],
                [
                    "0x7b4a2f",
                    "0x2faf080",
                    "0x5f5e100",
                ],
                [
                    "0x1f93413",
                    "0xdb58580",
                    "0x21d7ba0f",
                ],
                [
                    "0x54e0840",
                    "0x1dcd6500",
                    "0x379aa9ef",
                ],
                [
                    "0x14d05f48",
                    "0x1dcd6500",
                    "0x3b9aca03",
                ],
            ],
            baseFeePerGas: [
                "0x4a22b648",
                "0x46cfb436",
                "0x486359e3",
                "0x4c98640e",
                "0x48e6ecea",
                "0x44ee2016",
            ],
            gasUsedRatio: [
                0.3206394427510866,
                0.5890674544980715,
                0.7324794824330592,
                0.30713043651039335,
                0.28207058333333335,
            ],
            oldestBlock: "0x15a339f"
        )

        let calculator = GemFeeCalculator()

        let minPriorityFee: UInt64 = try calculator.calculateMinPriorityFee(
            gasUsedRatios: feeHisotry.gasUsedRatio,
            baseFee: feeHisotry.baseFeePerGas.last!,
            defaultMinPriorityFee: 1_000_000_000
        )

        let priorityFee = try calculator.calculatePriorityFees(
            feeHistory: feeHisotry,
            priorities: [.slow, .normal, .fast],
            minPriorityFee: minPriorityFee
        )

        print(minPriorityFee, priorityFee)
    }
}
