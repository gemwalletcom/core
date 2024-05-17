//
//  GemTestTests.swift
//  GemTestTests
//
//  Created by magician on 20/12/23.
//

import XCTest
import Gemstone
@testable import GemTest

extension Data {

    init?(hex: String) {
        guard hex.count.isMultiple(of: 2) else {
            return nil
        }

        let chars = hex.map { $0 }
        let bytes = stride(from: 0, to: chars.count, by: 2)
            .map { String(chars[$0]) + String(chars[$0 + 1]) }
            .compactMap { UInt8($0, radix: 16) }

        guard hex.count / bytes.count == 2 else { return nil }
        self.init(bytes)
      }

    func hexString() -> String {
        return map { String(format: "%02hhx", $0) }.joined()
    }
}

final class GemTestTests: XCTestCase {


    func testLoadFFI() async throws {
        let result = Gemstone.libVersion()
        XCTAssertFalse(result.isEmpty)
    }

    func testGetExplorerName() {
        let explorer = Explorer()
        XCTAssertEqual(explorer.getNameByHost(host: "etherscan.io"), "Etherscan")
        XCTAssertEqual(explorer.getNameByHost(host: "www.mintscan.io"), "MintScan")
    }

    func testSplitStake() throws {

        let input = SuiStakeInput(
            sender: "0xe6af80fe1b0b42fcd96762e5c70f5e8dae39f8f0ee0f118cac0d55b74e2927c2",
            validator: "0x61953ea72709eed72f4441dd944eec49a11b4acabfc8e04015e89c63be81b6ab",
            stakeAmount: 1_000_000_000,
            gas: SuiGas(
                budget: 20_000_000,
                price: 750
            ),
            coins: [SuiCoin(
                coinType: "0x2::sui::SUI",
                balance: 2_500_000_000,
                objectRef: SuiObjectRef(
                    objectId: "0x48a4367b1e0b4ad996375296e4471ae8d8df2576f90636b8e84051d97c1a363a",
                    digest: "9JHLrv8f3G3NFfWwWt54oHjdZYXD8VaNoZYXu3fV8pmB",
                    version: 65307031
                )
            )]
        )

        let output = try suiEncodeSplitStake(input: input)

        XCTAssertEqual(output.txData.hexString(), "000003000800ca9a3b0000000001010000000000000000000000000000000000000000000000000000000000000005010000000000000001002061953ea72709eed72f4441dd944eec49a11b4acabfc8e04015e89c63be81b6ab020200010100000000000000000000000000000000000000000000000000000000000000000000030a7375695f73797374656d11726571756573745f6164645f7374616b6500030101000300000000010200e6af80fe1b0b42fcd96762e5c70f5e8dae39f8f0ee0f118cac0d55b74e2927c20148a4367b1e0b4ad996375296e4471ae8d8df2576f90636b8e84051d97c1a363a9781e40300000000207b4ad8fc8964fdfbdf1c41f4c2ba993d607ae8ed85d260c500b8f4c5855ce0b6e6af80fe1b0b42fcd96762e5c70f5e8dae39f8f0ee0f118cac0d55b74e2927c2ee02000000000000002d31010000000000")
        XCTAssertEqual(output.hash.hexString(), "0437f7744c84f9226be8a09a76e9bedf4ca961f1937a354f72d5352266a72795")

    }

    func testConvertHRP() throws {
        let address = "cosmos1fxygpgus4nd5jmfl5j7fh5y8hyy53z8u95dzx7"

        XCTAssertEqual(try cosmosConvertHrp(address: address, hrp: "noble"), "noble1fxygpgus4nd5jmfl5j7fh5y8hyy53z8udhc27s")
    }

    func testDecodingBscDelegations() throws {
        let result = Data(hex:  "00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000001000000000000000000000000ee448667ffc3d15ca023a6deef2d0faf084c0716000000000000000000000000343da7ff0446247ca47aa41e2a25c5bbb230ed0a0000000000000000000000000000000000000000000000000e0932bb88351eef")!

        let delegations = try Gemstone.bscDecodeDelegationsReturn(result: result)

        XCTAssertEqual(delegations.count, 1)
        XCTAssertEqual(delegations[0].validatorAddress, "0x343dA7Ff0446247ca47AA41e2A25c5Bbb230ED0A")
        XCTAssertEqual(delegations[0].amount, "1011395372346842863")
    }
}
