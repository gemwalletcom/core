// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI
import Primitives
import Components
import Store
import PrimitivesComponents
import Localization
import InfoSheet

public struct ChartScene: View {
    @State private var model: ChartSceneViewModel

    public init(model: ChartSceneViewModel) {
        _model = State(initialValue: model)
    }
    
    public var body: some View {
        ChartListView(model: model) {
            if model.showPriceAlerts, let asset = model.priceData?.asset {
                NavigationLink(
                    value: Scenes.AssetPriceAlert(asset: asset),
                    label: {
                        ListItemView(
                            title: model.priceAlertsViewModel.priceAlertsTitle,
                            subtitle: model.priceAlertsViewModel.priceAlertCount
                        )
                    }
                )
            } else if model.isPriceAvailable {
                NavigationCustomLink(
                    with: ListItemView(
                        title: model.priceAlertsViewModel.setPriceAlertTitle
                    )
                ) {
                    model.onSelectSetPriceAlerts()
                }
            }

            if let priceDataModel = model.priceDataModel {
                marketSection(priceDataModel.marketValues)
                marketSection(priceDataModel.supplyValues)
                marketSection(priceDataModel.allTimeValues)

                if priceDataModel.showLinks {
                    Section(Localized.Social.links) {
                        SocialLinksView(model: priceDataModel.linksViewModel)
                    }
                }
            }
        }
        .bindQuery(model.priceQuery)
        .navigationTitle(model.title)
        .sheet(item: $model.isPresentingInfoSheet) {
            InfoSheetScene(type: $0)
        }
    }

    private func marketSection(_ items: [MarketValueViewModel]) -> some View {
        Section {
            ForEach(items, id: \.title) { item in
                if let url = item.url {
                    SafariNavigationLink(url: url) {
                        ListItemView(title: item.title, subtitle: item.subtitle)
                    }
                    .contextMenu(
                        item.value.map { [.copy(value: $0)] } ?? []
                    )
                } else {
                    ListItemView(
                        title: item.title,
                        titleTag: item.titleTag,
                        titleTagStyle: item.titleTagStyle ?? .body,
                        titleExtra: item.titleExtra,
                        subtitle: item.subtitle,
                        subtitleExtra: item.subtitleExtra,
                        subtitleStyleExtra: item.subtitleExtraStyle ?? .calloutSecondary,
                        infoAction: item.infoSheetType.map { type in { model.isPresentingInfoSheet = type } }
                    )
                }
            }
        }
    }
}
