// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import SwiftUI
import Primitives
import WalletConnectorService
import Localization
import PrimitivesComponents
import Components
import Style

public struct ConnectionProposalViewModel {
    private let confirmTransferDelegate: TransferDataCallback.ConfirmTransferDelegate
    private let pairingProposal: WCPairingProposal

    var walletSelectorModel: SelectWalletViewModel

    public init(
        confirmTransferDelegate: @escaping TransferDataCallback.ConfirmTransferDelegate,
        pairingProposal: WCPairingProposal
    ) {
        self.confirmTransferDelegate = confirmTransferDelegate
        self.pairingProposal = pairingProposal
        self.walletSelectorModel = SelectWalletViewModel(
            wallets: pairingProposal.proposal.wallets,
            selectedWallet: pairingProposal.proposal.defaultWallet
        )
    }
    
    var title: String { Localized.WalletConnect.Connect.title }
    var buttonTitle: String { Localized.Transfer.confirm }
    var walletTitle: String { Localized.Common.wallet }
    var appTitle: String { Localized.WalletConnect.app }
    var connectionTitle: String { Localized.WalletConnect.Connection.title }
    var connectionText: String { Localized.WalletConnect.brandName }

    var walletName: String {
        walletSelectorModel.selectedItems.first?.name ?? .empty
    }

    var appName: String {
        payload.metadata.shortName
    }
    
    var websiteText: String? {
        guard let url = URL(string: payload.metadata.url), let host = url.host(percentEncoded: true) else {
            return .none
        }
        return host
    }
    
    var appText: String {
        AppDisplayFormatter.format(name: appName, host: websiteText)
    }
    
    var imageUrl: URL? {
        URL(string: payload.metadata.icon)
    }

    var verificationImage: Image {
        switch pairingProposal.verificationStatus {
        case .verified: Images.Transaction.State.success
        case .unknown: Images.TokenStatus.warning
        case .invalid, .malicious: Images.TokenStatus.risk
        }
    }

    var statusText: String {
        switch pairingProposal.verificationStatus {
        case .verified: Localized.Asset.Verification.verified
        case .unknown: Localized.Asset.Verification.unverified
        case .invalid, .malicious: Localized.Asset.Verification.suspicious
        }
    }

    var statusTextStyle: TextStyle {
        switch pairingProposal.verificationStatus {
        case .verified: TextStyle(font: .callout, color: Colors.green)
        case .unknown: TextStyle(font: .callout, color: Colors.orange)
        case .invalid, .malicious: TextStyle(font: .callout, color: Colors.red)
        }
    }

    var statusAssetImage: AssetImage {
        .image(verificationImage)
    }

    var permissionsTitle: String { Localized.WalletConnect.Permissions.title }

    var permissions: [ListItemModel] {
        [
            ListItemModel(
                title: Localized.WalletConnect.Permissions.viewBalance,
                imageStyle: .accessory(assetImage: .image(Images.System.checkmark), fontWeight: .semibold)
            ),
            ListItemModel(
                title: Localized.WalletConnect.Permissions.approvalRequests,
                imageStyle: .accessory(assetImage: .image(Images.System.checkmark), fontWeight: .semibold)
            ),
        ]
    }

    var appPreview: AppPreviewModel {
        AppPreviewModel(
            assetImage: AssetImage(imageURL: imageUrl),
            name: appName,
            subtitleSymbol: websiteText
        )
    }

    private var payload: WalletConnectionSessionProposal {
        pairingProposal.proposal
    }
}

// MARK: - Business Logic

extension ConnectionProposalViewModel {
    func accept() throws {
        guard let selectedWallet = walletSelectorModel.selectedItems.first else {
            return
        }
        confirmTransferDelegate(.success(selectedWallet.id))
    }
}
