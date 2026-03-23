// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import SwiftUI
import Blockchain
import Components
import PrimitivesComponents
import Style
import Localization
import ChainService
import GemstonePrimitives

@Observable
@MainActor
public final class AddAssetSceneViewModel {
    private let service: any AddAssetServiceable

    var state: StateViewType<AddAssetViewModel> = .noData
    var input: AddAssetInput

    var isPresentingScanner = false

    public init(wallet: Wallet, service: any AddAssetServiceable) {
        self.service = service
        self.input = AddAssetInput(chains: wallet.chainsWithTokens)
    }

    var title: String { Localized.Wallet.AddToken.title }
    var networkTitle: String { Localized.Transfer.network }
    var errorTitle: String { Localized.Errors.errorOccured }
    var actionButtonTitle: String { Localized.Wallet.Import.action }
    var addressTitleField: String { Localized.Wallet.Import.contractAddressField }

    var pasteImage: Image { Images.System.paste }
    var qrImage: Image { Images.System.qrCodeViewfinder }
    var errorSystemImage: String { SystemImage.errorOccurred }

    var chains: [Chain] { input.chains }

    var addressBinding: Binding<String> {
        Binding(
            get: { [self] in
                self.input.address ?? ""
            },
            set: { [self] in
                self.input.address = $0.isEmpty ? nil : $0
            }
        )
    }
    
    var warningImageStyle: ListItemImageStyle? {
        ListItemImageStyle(
            assetImage: AssetImage(type: Emoji.WalletAvatar.warning.rawValue),
            imageSize: .image.semiMedium,
            alignment: .top,
            cornerRadiusType: .none
        )
    }
    
    var tokenVerificationUrl: URL {
        Docs.url(.tokenVerification)
    }
    var customTokenUrl: URL {
        Docs.url(.addCustomToken)
    }
}

// MARK: - Business Logic

extension AddAssetSceneViewModel {
    func fetch() async {
        guard let chain = input.chain, let address = input.address, !address.isEmpty else {
            state = .noData
            return
        }

        state = .loading

        do {
            let asset = try await service.getTokenData(chain: chain, tokenId: address)
            state = .data(AddAssetViewModel(asset: asset))
        } catch {
            state.setError(error)
        }
    }
}
