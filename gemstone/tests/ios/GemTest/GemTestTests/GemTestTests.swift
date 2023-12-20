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
        
        let result = Gemstone.add(a: 4, b: 2)
        XCTAssertEqual(result, 6)
        
        let obj = Gemstone.addObj(a: RustDemoObj(value: 1), b: RustDemoObj(value: 2))
        XCTAssertEqual(obj, RustDemoObj(value: 3))
        
        let string = await Gemstone.sayAfter(ms: 500, who: "Async")
        XCTAssertTrue(string.contains("Async"))
    }
}
