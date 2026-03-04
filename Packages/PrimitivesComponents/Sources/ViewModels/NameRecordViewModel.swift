// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

@Observable
@MainActor
public final class NameRecordViewModel {
    private let nameService: any NameServiceable
    private var resolveTask: Task<Void, Never>?

    public var state: NameRecordState = .none

    public init(nameService: any NameServiceable) {
        self.nameService = nameService
    }

    public func resolve(name: String, chain: Chain) {
        resolveTask?.cancel()

        guard nameService.canResolveName(name: name) else {
            state = .none
            return
        }

        state = .loading
        resolveTask = Task {
            do {
                try await Task.sleep(for: .debounce)
                if let record = try await nameService.getName(name: name, chain: chain.rawValue) {
                    state = .complete(record)
                } else {
                    state = .error
                }
            } catch {
                if !error.isCancelled {
                    state = .error
                }
            }
        }
    }

    public func reset() {
        resolveTask?.cancel()
        state = .none
    }

    public func canResolveName(name: String) -> Bool {
        nameService.canResolveName(name: name)
    }
}
