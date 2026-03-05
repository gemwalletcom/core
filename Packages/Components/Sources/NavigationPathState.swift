// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI

@Observable
@MainActor
public final class NavigationPathState {
    private static let encoder = JSONEncoder()
    private static let decoder = JSONDecoder()

    private(set) public var path = NavigationPath()

    public var binding: Binding<NavigationPath> {
        Binding(
            get: { self.path },
            set: { self.path = $0 }
        )
    }

    public var count: Int { path.count }
    public var isEmpty: Bool { path.isEmpty }

    public init() {}

    @discardableResult
    public func append<T: Hashable & Codable>(_ value: T) -> Bool {
        if isLastElement(value) {
            return false
        }
        path.append(value)
        return true
    }

    public func setPath(_ items: [any Hashable & Codable]) {
        path = items.reduce(into: NavigationPath()) { $0.append($1) }
    }

    public func reset() {
        guard !path.isEmpty else { return }
        path.removeLast(path.count)
    }

    public func removeLast(_ k: Int = 1) {
        path.removeLast(k)
    }
}

// MARK: - Private

extension NavigationPathState {
    private func encodedElements(_ codable: NavigationPath.CodableRepresentation) -> [String] {
        guard let data = try? Self.encoder.encode(codable),
              let elements = try? Self.decoder.decode([String].self, from: data) else {
            return []
        }
        return elements
    }

    private func isLastElement<T: Hashable & Codable>(_ value: T) -> Bool {
        guard !path.isEmpty, let currentCodable = path.codable else { return false }

        let currentElements = encodedElements(currentCodable)

        guard currentElements.count >= 2,
              currentElements[0] == String(reflecting: T.self),
              let data = currentElements[1].data(using: .utf8),
              let lastValue = try? Self.decoder.decode(T.self, from: data) else {
            return false
        }

        return lastValue == value
    }
}
