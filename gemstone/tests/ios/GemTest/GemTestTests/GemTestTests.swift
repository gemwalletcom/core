//
//  GemTestTests.swift
//  GemTestTests
//
//  Created by magician on 20/12/23.
//

import XCTest
import Gemstone
@testable import GemTest

final class GemTestTests: XCTestCase {

    func testLoadFFI() async throws {
        let result = Gemstone.libVersion()
        XCTAssertFalse(result.isEmpty)
                
        let string = await Gemstone.sayAfter(ms: 500, who: "Async")
        XCTAssertTrue(string.contains("Async"))
    }
    
    func testGetExplorerName() {
        XCTAssertEqual(Gemstone.getNameByHost(host: "etherscan.io"), "Etherscan")
        XCTAssertEqual(Gemstone.getNameByHost(host: "www.mintscan.io"), "MintScan")
    }
}
