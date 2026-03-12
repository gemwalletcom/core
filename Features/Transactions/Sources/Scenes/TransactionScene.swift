// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import SwiftUI
import Components
import Style
import PrimitivesComponents

public struct TransactionScene: View {
    private let model: TransactionSceneViewModel

    public init(model: TransactionSceneViewModel) {
        self.model = model
    }

    public var body: some View {
        ListSectionView(
            provider: model,
            content: content(for:)
        )
        .contentMargins([.top], .small, for: .scrollContent)
        .listSectionSpacing(.compact)
        .background(Colors.grayBackground)
        .navigationTitle(model.title)
    }

    @ViewBuilder
    private func content(for itemModel: TransactionItemModel) -> some View {
        switch itemModel {
        case let .listItem(model):
            ListItemView(model: model)
        case let .fee(model):
            NavigationCustomLink(
                with: ListItemView(model: model),
                action: self.model.onSelectFeeDetails
            )
        case let .header(model):
            TransactionHeaderListItemView(
                model: model,
                action: self.model.onSelectTransactionHeader
            )
        case let .participant(model):
            AddressListItemView(model: model.addressViewModel)
        case let .network(title, subtitle, image):
            ListItemImageView(
                title: title,
                subtitle: subtitle,
                assetImage: image
            )
        case let .pnl(title, value, color):
            ListItemView(
                title: title,
                subtitle: value,
                subtitleStyle: TextStyle(font: .callout, color: color)
            )
        case let .price(title, value):
            ListItemView(
                title: title,
                subtitle: value
            )
        case let .explorer(url, text):
            SafariNavigationLink(url: url) {
                Text(text)
                    .tint(Colors.black)
            }
        case let .swapAgain(text):
            let button = StateButton(
                text: text,
                type: .primary(.normal),
                action: model.onSelectTransactionHeader
            )
            .cleanListRow(topOffset: .zero)
            if #available(iOS 26, *) {
                button.cornerRadius(.scene.button.height / 2) // TODO: - Think about what to do with this button
            }
        case .empty:
            EmptyView()
        }
    }
}
