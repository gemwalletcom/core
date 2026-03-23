// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI

private struct TimerModifier<ID: Equatable & Sendable>: ViewModifier {
    let interval: TimeInterval
    let id: ID
    let action: @Sendable () async -> Void

    func body(content: Content) -> some View {
        content
            .task(id: id) {
                while !Task.isCancelled {
                    try? await Task.sleep(for: .seconds(interval))
                    guard !Task.isCancelled else { break }
                    await action()
                }
            }
    }
}

public extension View {
    func onTimer(every interval: TimeInterval, action: @Sendable @escaping () async -> Void) -> some View {
        modifier(TimerModifier(interval: interval, id: 0, action: action))
    }

    func onTimer<ID: Equatable & Sendable>(every interval: TimeInterval, id: ID, action: @Sendable @escaping () async -> Void) -> some View {
        modifier(TimerModifier(interval: interval, id: id, action: action))
    }
}
