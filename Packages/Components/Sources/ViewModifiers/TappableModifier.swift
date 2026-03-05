// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI

private struct TappableModifier: ViewModifier {
    let action: () -> Void

    func body(content: Content) -> some View {
        content
            .contentShape(Rectangle())
            .onTapGesture(perform: action)
    }
}

public extension View {
    func onTap(action: @escaping () -> Void) -> some View {
        modifier(TappableModifier(action: action))
    }
}
