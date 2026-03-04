// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import PrimitivesTestKit
@testable import NameService

public extension NameServiceable where Self == MockNameService {
    static func mock(
        canResolve: Bool = true,
        nameRecord: NameRecord? = nil
    ) -> MockNameService {
        MockNameService(canResolve: canResolve, nameRecord: nameRecord)
    }
}

public struct MockNameService: NameServiceable {
    let canResolve: Bool
    let nameRecord: NameRecord?
    
    public init(canResolve: Bool = true, nameRecord: NameRecord? = nil) {
        self.canResolve = canResolve
        self.nameRecord = nameRecord
    }
    
    public func getName(name: String, chain: String) async throws -> NameRecord? {
        nameRecord ?? NameRecord.mock()
    }
}
