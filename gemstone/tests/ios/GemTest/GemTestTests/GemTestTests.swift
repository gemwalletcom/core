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
    func hexString() -> String {
        return map { String(format: "%02hhx", $0) }.joined()
    }
}

final class GemTestTests: XCTestCase {


    
    func testLoadFFI() async throws {
        let result = Gemstone.libVersion()
        XCTAssertFalse(result.isEmpty)
                
        let string = await Gemstone.sayAfter(ms: 500, who: "Async")
        XCTAssertTrue(string.contains("Async"))
    }
    
    func testGetExplorerName() {
        XCTAssertEqual(explorerGetNameByHost(host: "etherscan.io"), "Etherscan")
        XCTAssertEqual(explorerGetNameByHost(host: "www.mintscan.io"), "MintScan")
    }
    
    func testSplitStake() throws {
        
        let input = SuiStakeInput(
            sender: "0xe6af80fe1b0b42fcd96762e5c70f5e8dae39f8f0ee0f118cac0d55b74e2927c2",
            toValidator: "0x61953ea72709eed72f4441dd944eec49a11b4acabfc8e04015e89c63be81b6ab",
            stakeAmount: 1_000_000_000,
            gasBudget: 20_000_000,
            gasPrice: 750,
            coin: SuiCoin(
                coinType: "0x2::coin::Coin<0x2::sui::SUI>",
                coinObjectId: "0x48a4367b1e0b4ad996375296e4471ae8d8df2576f90636b8e84051d97c1a363a",
                version: 65307031,
                digest: "9JHLrv8f3G3NFfWwWt54oHjdZYXD8VaNoZYXu3fV8pmB",
                balance: 2_500_000_000
            )
        )
        
        let encoded = try suiEncodeSplitStake(input: input)
        
        XCTAssertEqual(encoded.hexString(), "000000000003000800ca9a3b00000000010100000000000000000000000000000000000000000000000000000000000000050100000000000000010020e6af80fe1b0b42fcd96762e5c70f5e8dae39f8f0ee0f118cac0d55b74e2927c2020200010100000000000000000000000000000000000000000000000000000000000000000000030a7375695f73797374656d11726571756573745f6164645f7374616b6500030101000300000000010200e6af80fe1b0b42fcd96762e5c70f5e8dae39f8f0ee0f118cac0d55b74e2927c20148a4367b1e0b4ad996375296e4471ae8d8df2576f90636b8e84051d97c1a363a9781e40300000000207b4ad8fc8964fdfbdf1c41f4c2ba993d607ae8ed85d260c500b8f4c5855ce0b6e6af80fe1b0b42fcd96762e5c70f5e8dae39f8f0ee0f118cac0d55b74e2927c2ee02000000000000002d31010000000000")
        
    }
}
