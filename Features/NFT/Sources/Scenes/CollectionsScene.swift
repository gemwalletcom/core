// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import SwiftUI
import Primitives
import Store
import Components
import Style
import PrimitivesComponents
import Localization

public struct CollectionsScene<ViewModel: CollectionsViewable>: View {
    @State private var model: ViewModel

    public init(model: ViewModel) {
        _model = State(initialValue: model)
    }

    public var body: some View {
        GeometryReader { geometry in
            ScrollView {
                VStack(spacing: .zero) {
                    LazyVGrid(columns: model.columns) {
                        collectionsView
                    }
                    .padding(.horizontal, .medium)

                    Spacer(minLength: .medium)

                    if let unverifiedCount = model.content.unverifiedCount {
                        List {
                            NavigationLink(value: Scenes.UnverifiedCollections()) {
                                ListItemView(
                                    title: Localized.Asset.Verification.unverified,
                                    subtitle: unverifiedCount
                                )
                            }
                        }
                        .scrollDisabled(true)
                        .frame(height: .list.minHeight)
                    }
                }
                .frame(minHeight: geometry.size.height)
            }
        }
        .bindQuery(model.query)
        .overlay {
            if model.content.items.isEmpty {
                EmptyContentView(model: model.emptyContentModel)
            }
        }
        .background { Colors.insetGroupedListStyle.ignoresSafeArea() }
        .navigationBarTitleDisplayMode(.inline)
        .navigationTitle(model.title)
        .refreshable { await model.fetch() }
        .task { await model.fetch() }
    }
}

// MARK: - UI

extension CollectionsScene {
    private var collectionsView: some View {
        ForEach(model.content.items) { item in
            NavigationLink(value: item.destination) {
                GridPosterView(model: item.model)
            }
        }
    }
}
