// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Formatters
import Localization
import Primitives
import PrimitivesComponents
import Foundation

public struct ConfirmAppViewModel: ItemModelProvidable {
    private let type: TransferDataType

    init(type: TransferDataType) {
        self.type = type
    }

    var websiteURL: URL? {
        switch type {
        case .transfer,
                .deposit,
                .withdrawal,
                .transferNft,
                .swap,
                .tokenApprove,
                .stake,
                .account,
                .perpetual,
                .earn: .none
        case .generic(_, let metadata, _):
            URL(string: metadata.url)
        }
    }

    var websiteTitle: String { Localized.Settings.website  }
}

// MARK: - ItemModelPrividable

extension ConfirmAppViewModel {
    public var itemModel: ConfirmTransferItemModel {
        guard let name = appValue else { return .empty }

        let subtitle = AppDisplayFormatter.format(
            name: name,
            host: websiteURL?.cleanHost()
        )

        return .app(
            ListItemModel(
                title: Localized.WalletConnect.app,
                subtitle: subtitle,
                imageStyle: .list(assetImage: assetImage)
            )
        )
    }
}

// MARK: - Private

extension ConfirmAppViewModel {
    private var appValue: String? {
        switch type {
        case .transfer,
                .deposit,
                .withdrawal,
                .transferNft,
                .swap,
                .tokenApprove,
                .stake,
                .account,
                .perpetual,
                .earn: .none
        case .generic(_, let metadata, _):
            metadata.shortName
        }
    }

    private var assetImage: AssetImage? {
        switch type {
        case .transfer,
                .deposit,
                .withdrawal,
                .transferNft,
                .swap,
                .tokenApprove,
                .stake,
                .account,
                .perpetual,
                .earn:
                .none
        case let .generic(_, session, _):
            AssetImage(imageURL: session.icon.asURL)
        }
    }
}
