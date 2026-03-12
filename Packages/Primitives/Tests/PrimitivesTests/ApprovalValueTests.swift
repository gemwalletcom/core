import Testing
import BigInt
import PrimitivesTestKit
@testable import Primitives

struct ApprovalValueTests {
    @Test
    func initRawValue() {
        #expect(ApprovalValue(rawValue: "Unlimited") == .unlimited)
        #expect(ApprovalValue(rawValue: "1000000") == .exact(BigInt(1000000)))
        #expect(ApprovalValue(rawValue: "invalid") == nil)
    }

    @Test
    func rawValue() {
        #expect(ApprovalValue.unlimited.rawValue == "Unlimited")
        #expect(ApprovalValue.exact(BigInt(42)).rawValue == "42")
    }
}
