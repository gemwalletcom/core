// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GemAPI
import Primitives

public final class GemAPINFTServiceMock: GemAPINFTService, @unchecked Sendable {
    public init() {}

    public func getDeviceNFTAssets(walletId: String) async throws -> [NFTData] { [] }

    public func reportNft(report: ReportNft) async throws {}
}
