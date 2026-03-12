// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI
import Style
import Localization

public extension View {
    func contextMenu(_ items: [ContextMenuItemType]) -> some View {
        self.contextMenu {
            ForEach(Array(items.enumerated()), id: \.offset) {
                build($0.element)
            }
        }
    }

    func contextMenu(_ item: ContextMenuItemType) -> some View {
        contextMenu([item])
    }

    @ViewBuilder
    private func build(_ item: ContextMenuItemType) -> some View {
        switch item {
        case let .copy(title, value, expirationTime, onCopied):
            ContextMenuItem(
                title: title ?? Localized.Common.copy,
                systemImage: SystemImage.copy
            ) {
                CopyTypeViewModel.copyToClipboard(value, expirationTime: expirationTime)
                onCopied?(value)
            }
        case let .pin(isPinned, onPin):
            ContextMenuItem(
                title: isPinned ? Localized.Common.unpin : Localized.Common.pin,
                systemImage: isPinned ? SystemImage.unpin : SystemImage.pin,
                action: {
                    onPin?()
                }
            )
        case let .hide(onHide):
            ContextMenuItem(
                title: Localized.Common.hide,
                systemImage: SystemImage.hide,
                action: {
                    onHide?()
                }
            )
        case let .delete(onDelete):
            ContextMenuItem(
                title: Localized.Common.delete,
                systemImage: SystemImage.delete,
                role: .destructive,
                action: {
                    onDelete?()
                }
            )
        case let .url(title, onOpen):
            ContextMenuItem(
                title: title,
                systemImage: SystemImage.globe
            ) {
                onOpen?()
            }
        case let .custom(title, systemImage, role, action):
            ContextMenuItem(
                title: title,
                systemImage: systemImage,
                role: role
            ) {
                action?()
            }
        }
    }
}
