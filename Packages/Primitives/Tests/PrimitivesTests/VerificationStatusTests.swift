import Testing
@testable import Primitives

struct VerificationStatusTests {
    @Test
    func assetScoreType() {
        #expect(AssetScoreType(verificationStatus: .verified) == .verified)
        #expect(AssetScoreType(verificationStatus: .unverified) == .unverified)
        #expect(AssetScoreType(verificationStatus: .suspicious) == .suspicious)
    }
}
