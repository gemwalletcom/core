// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI
import Primitives
import Components
import Style
import Localization

public struct AddressListItemView: View {
    @State private var isPresentingUrl: URL? = nil
    @State private var showAddress: Bool = false
    private let model: AddressListItemViewModel

    public init(model: AddressListItemViewModel) {
        self.model = model
    }

    public var body: some View {
        ListItemImageView(
            title: model.title,
            subtitle: showAddress ? model.addressSubtitle : model.subtitle,
            assetImage: model.assetImage,
            assetImageStyle: model.assetImageStyle,
            imageSize: model.assetImageSize
        )
        .onTap {
            if model.canToggleAddress {
                showAddress.toggle()
            }
        }
        .contextMenu([
            .copy(value: model.account.address),
            .url(title: model.addressExplorerText, onOpen: { isPresentingUrl = model.addressExplorerUrl })
        ])
        .safariSheet(url: $isPresentingUrl)
    }
}
